use std::env::current_dir;
use std::fs;

use image::RgbaImage;
use wgpu::{Origin3d, TextureAspect};

use super::bind_group;

const TILE_SIZE: usize = 1024;

pub struct Material {
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    fn tile_image(image_bytes: &[u8], tiling: bool) -> Vec<(u32, u32, RgbaImage)> {
        let loaded_image =
            image::load_from_memory(&image_bytes).expect("Could not Read Image from bytes");
        let converted = loaded_image.to_rgba8();
        let (width, height) = converted.dimensions();
        let tile_size = TILE_SIZE as u32;
        let mut tiles: Vec<(u32, u32, RgbaImage)> = vec![];

        if !tiling {
            let tile_width = tile_size.min(width);
            let tile_height = tile_size.min(height);
            let tile =
                image::imageops::crop_imm(&converted, 0, 0, tile_width, tile_height).to_image();
            tiles.push((0, 0, tile));
            return tiles;
        }

        if width <= tile_size && height <= tile_size {
            tiles.push((0, 0, converted));
            return tiles;
        }

        for y in (0..height).step_by(tile_size as usize) {
            for x in (0..width).step_by(tile_size as usize) {
                let tile = image::imageops::crop_imm(
                    &converted,
                    x,
                    y,
                    (x + tile_size).min(width) - x,
                    (y + tile_size).min(height) - y,
                )
                .to_image();
                tiles.push((x, y, tile));
            }
        }

        tiles
    }

    pub fn new(
        filename: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &str,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let mut filepath = current_dir().unwrap();
        filepath.push(filename);
        let mut errmsg = String::from("Couldn't Read Image File from File path - ");
        errmsg.push_str(filepath.as_os_str().to_str().unwrap());
        let bytes = fs::read(filepath).expect(&errmsg);

        let tiles = Self::tile_image(&bytes, false);
        let texture_data = &tiles.get(0).expect("No Tiles available").2;
        let (width, height) = texture_data.dimensions();

        // let loaded_image = image::load_from_memory(&bytes).expect(&errmsg);
        // let converted = loaded_image.to_rgb8();
        // let size = loaded_image.dimensions();

        let texture_size = wgpu::Extent3d {
            depth_or_array_layers: 1,
            width,
            height,
        };

        let texture_descriptor = wgpu::TextureDescriptor {
            label: Some(filename),
            mip_level_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            sample_count: 1,
            size: texture_size,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        };

        let texture = device.create_texture(&texture_descriptor);

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            texture_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        let view_descriptor = wgpu::TextureViewDescriptor {
            ..Default::default()
        };
        let view = texture.create_view(&view_descriptor);

        let sampler_descriptor = wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        };
        let sampler = device.create_sampler(&sampler_descriptor);

        let mut builder = bind_group::Builder::new(device);
        builder.set_layout(layout);
        builder.add_material(&view, &sampler);
        let bind_group = builder.build(label);

        Material { bind_group }
    }
}
