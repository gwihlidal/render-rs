//use strum::EnumCount;
use enum_primitive::FromPrimitive;
use render_core::format::build_format;
use render_core::format::channel_format_component_count;
use render_core::format::channel_format_has_alpha;
use render_core::format::channel_format_has_depth;
use render_core::format::channel_format_has_stencil;
use render_core::format::channel_format_is_compressed;
use render_core::format::channel_format_min_dimensions;
use render_core::format::format_has_depth;
use render_core::format::format_has_stencil;
use render_core::types::RenderChannelFormat;
use render_core::types::RenderFormat;
use render_core::types::RenderNumericFormat;
use render_core::types::RENDERCHANNELFORMAT_COUNT;
use render_core::types::RENDERNUMERICFORMAT_COUNT;

#[test]
fn formats() {
    for format_index in 0..RENDERCHANNELFORMAT_COUNT {
        let channel_format = RenderChannelFormat::from_u32(format_index as u32).unwrap();
        if channel_format != RenderChannelFormat::Unknown {
            assert!(channel_format_component_count(channel_format) >= 1);
        }

        for numeric_index in 0..RENDERNUMERICFORMAT_COUNT {
            let numeric_format = RenderNumericFormat::from_u32(numeric_index as u32).unwrap();

            let format = build_format(channel_format, numeric_format, false);
            if format == RenderFormat::Unknown {
                continue;
            }

            let channel_check: RenderChannelFormat = format.into();
            let numeric_check: RenderNumericFormat = format.into();
            assert_eq!(channel_check, channel_format);
            assert_eq!(numeric_check, numeric_format);

            match channel_format {
                RenderChannelFormat::D24S8
                | RenderChannelFormat::D32S8
                | RenderChannelFormat::D16
                | RenderChannelFormat::D24
                | RenderChannelFormat::D32 => {
                    assert_eq!(channel_format_has_depth(channel_format), true);
                    assert_eq!(format_has_depth(format), true);
                    assert_eq!(channel_format_is_compressed(channel_format), false);
                    assert_eq!(channel_format_has_alpha(channel_format), false);
                }
                _ => {
                    assert_eq!(channel_format_has_depth(channel_format), false);
                    assert_eq!(format_has_depth(format), false);
                    assert_eq!(channel_format_has_stencil(channel_format), false);
                    assert_eq!(format_has_stencil(format), false);
                }
            }

            let (min_width, min_height) = channel_format_min_dimensions(channel_format);
            assert!(min_width >= 1);
            assert!(min_height >= 1);
        }
    }
}
