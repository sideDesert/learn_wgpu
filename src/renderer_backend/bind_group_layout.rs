pub struct Builder<'builder> {
    entries: Vec<wgpu::BindGroupLayoutEntry>,
    device: &'builder wgpu::Device,
}

impl<'builder> Builder<'builder> {
    pub fn new(device: &'builder wgpu::Device) -> Self {
        Self {
            entries: Vec::new(),
            device,
        }
    }

    fn reset(&mut self) {
        self.entries.clear();
    }

    pub fn add_material(&mut self) {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.entries.len() as u32,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        });

        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.entries.len() as u32,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });
    }

    pub fn build(&mut self, label: &str) -> wgpu::BindGroupLayout {
        let desc = wgpu::BindGroupLayoutDescriptor {
            label: Some(label),
            entries: &self.entries,
        };
        let layout = self.device.create_bind_group_layout(&desc);
        self.reset();

        layout
    }
}
