use ash;
use render_core::utilities::align_forward;
use render_core::utilities::is_aligned;
use vk_mem;

const DEFAULT_ALIGNMENT: usize = 256;
const MAX_ALIGNMENT: usize = 64 * 1024;

struct ResourceBlock {
    buffer: ash::vk::Buffer,
    allocation: vk_mem::Allocation,
    allocation_info: vk_mem::AllocationInfo,
    current_offset: usize,
    //allocation_count: usize,
}

pub struct HostLinearAllocation {
    /// Buffer we're pointing to.
    pub buffer: ash::vk::Buffer,

    /// Device memory.
    pub device_memory: ash::vk::DeviceMemory,

    /// Offset.
    pub offset: usize,

    /// Address of allocation.
    pub address: *mut u8,

    /// Size of allocation.
    pub size: usize,
}

pub struct HostLinearAllocator {
    blocks: Vec<ResourceBlock>,
    blocks_created: usize,
    min_resource_block_size: usize,
    pub(crate) internal: vk_mem::Allocator,
}

impl HostLinearAllocator {
    pub fn new(
        physical_device: ash::vk::PhysicalDevice,
        device: ash::Device,
        instance: ash::Instance,
        min_resource_block_size: usize,
    ) -> Self {
        use ash::vk::Handle;
        let create_info = vk_mem::AllocatorCreateInfo {
            physical_device,
            device,
            instance,
            ..Default::default()
        };
        HostLinearAllocator {
            blocks: Vec::new(),
            blocks_created: 0,
            min_resource_block_size,
            internal: vk_mem::Allocator::new(&create_info).unwrap(),
        }
    }

    #[inline(always)]
    fn free_block_index(&self, size: usize, alignment: usize) -> Option<usize> {
        // Check if we have a valid block size.
        for block_index in 0..self.blocks.len() {
            let block = &self.blocks[block_index];
            let aligned_offset = align_forward(block.current_offset, alignment);
            let remaining = block.allocation_info.get_size() as i64 - aligned_offset as i64;
            if remaining >= size as i64 {
                return Some(block_index);
            }
        }

        None
    }

    pub fn allocate(&mut self, size: usize, alignment: Option<usize>) -> HostLinearAllocation {
        let alignment = alignment.unwrap_or(DEFAULT_ALIGNMENT);
        assert!(alignment <= MAX_ALIGNMENT);
        let block_index = self.free_block_index(size, alignment).unwrap_or_else(|| {
            let new_block = self.create_resource_block(size);
            self.blocks.push(new_block);
            self.remap();
            self.blocks.len() - 1
        });

        let mut block = &mut self.blocks[block_index];

        // Grab the correct offset and assert remaining size
        let aligned_offset = align_forward(block.current_offset, alignment);
        let remaining = block.allocation_info.get_size() as i64 - aligned_offset as i64;
        assert!(remaining >= size as i64);

        // Advance current offset.
        block.current_offset = aligned_offset + size;
        //assert_eq!(is_aligned(block.current_offset, alignment), true);
        assert_ne!(
            block.allocation_info.get_mapped_data(),
            std::ptr::null_mut()
        );

        HostLinearAllocation {
            buffer: block.buffer.clone(),
            device_memory: block.allocation_info.get_device_memory(),
            offset: aligned_offset,
            address: unsafe {
                block
                    .allocation_info
                    .get_mapped_data()
                    .offset(aligned_offset as isize)
            },
            size,
        }
    }

    pub fn reset(&mut self) {
        let mut total_usage = 0;
        let mut total_size = 0;
        for block in &mut self.blocks {
            total_usage += block.current_offset;
            total_size += block.allocation_info.get_size();
            block.current_offset = 0;
        }

        // If we have many blocks, merge into 1 and create as large as current total size.
        let use_compaction = true;
        if use_compaction {
            if self.blocks.len() > 1 {
                println!(
                    "Compacting linear heap allocator - size: {} / {} Kb. Blocks: {} / {}",
                    total_size / 1024,
                    total_usage / 1024,
                    self.blocks_created,
                    self.blocks.len()
                );

                for block in &self.blocks {
                    self.internal
                        .destroy_buffer(block.buffer, &block.allocation)
                        .unwrap();
                }

                let new_block = self.create_resource_block(total_size);
                self.blocks.clear();
                self.blocks.push(new_block);
            }
        }

        self.remap();
        self.blocks_created = 0;
    }

    pub fn remap(&mut self) {
        assert_eq!(self.active_block_count(), 0); // Invalid to call remap with activation allocations!
        for block in &mut self.blocks {
            self.internal
                .get_allocation_info(&mut block.allocation)
                .unwrap();
        }
    }

    pub fn active_block_count(&self) -> usize {
        let mut count = 0;
        for block in &self.blocks {
            count += if block.current_offset > 0 { 1 } else { 0 };
        }
        count
    }

    fn create_resource_block(&mut self, size: usize) -> ResourceBlock {
        // Minimum sized block, round up to max alignment.
        let block_size = align_forward(
            std::cmp::max(size, self.min_resource_block_size),
            MAX_ALIGNMENT,
        );
        let buffer_info = ash::vk::BufferCreateInfo::builder()
            .size(block_size as u64)
            .usage(ash::vk::BufferUsageFlags::TRANSFER_SRC)
            .build();
        let allocation_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::CpuToGpu,
            flags: vk_mem::AllocationCreateFlags::MAPPED,
            ..Default::default()
        };
        let (buffer, allocation, allocation_info) = self
            .internal
            .create_buffer(&buffer_info, &allocation_info)
            .unwrap();
        self.blocks_created += 1;
        ResourceBlock {
            buffer,
            allocation,
            allocation_info,
            current_offset: 0,
            //allocation_count: 0,
        }
    }
}

impl Drop for HostLinearAllocator {
    fn drop(&mut self) {
        for block in &self.blocks {
            self.internal
                .destroy_buffer(block.buffer, &block.allocation)
                .unwrap();
        }
    }
}
