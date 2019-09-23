export SDK_PATH=/Users/graham/src/github.com/gwihlidal/render-rs/redist/vulkansdk-macos-1.1.85.0
export DYLD_LIBRARY_PATH=$SDK_PATH/macOS/lib
export VK_ICD_FILENAMES=$SDK_PATH/macOS/etc/vulkan/icd.d/MoltenVK_icd.json
export VK_LAYER_PATH=$SDK_PATH/macOS/etc/vulkan/explicit_layer.d

cargo +stable fmt
cargo build --all
cargo run --bin debug