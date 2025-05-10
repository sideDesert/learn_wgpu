pub struct Builder<'builder> {
    entries: Vec<wgpu::BindGroupEntry<'builder>>,
    layout: Option<&'builder wgpu::BindGroupLayout>,
    device: &'builder wgpu::Device,
}

impl<'builder> Builder<'builder> {
    pub fn new(device: &'builder wgpu::Device) -> Self {
        Self {
            entries: Vec::new(),
            layout: None,
            device,
        }
    }

    pub fn set_layout(&mut self, layout: &'builder wgpu::BindGroupLayout) {
        self.layout = Some(layout);
    }

    fn get_layout(&self) -> &'builder wgpu::BindGroupLayout {
        self.layout.as_ref().unwrap()
    }

    fn reset(&mut self) {
        self.entries.clear();
    }

    pub fn add_material(
        &mut self,
        view: &'builder wgpu::TextureView,
        sampler: &'builder wgpu::Sampler,
    ) {
        self.entries.push(wgpu::BindGroupEntry {
            binding: self.entries.len() as u32,
            resource: wgpu::BindingResource::TextureView(view),
        });

        self.entries.push(wgpu::BindGroupEntry {
            binding: self.entries.len() as u32,
            resource: wgpu::BindingResource::Sampler(sampler),
        });
    }

    pub fn build(&mut self, label: &str) -> wgpu::BindGroup {
        let desc = wgpu::BindGroupDescriptor {
            label: Some(label),
            entries: &self.entries,
            layout: self.get_layout(),
        };
        let bind_group = self.device.create_bind_group(&desc);
        self.reset();

        bind_group
    }
}
