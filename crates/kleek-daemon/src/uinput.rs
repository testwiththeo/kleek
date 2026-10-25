use anyhow::Result;

#[derive(Debug, Clone)]
pub struct VirtualMouseBuilder {
    supports_hi_res: bool,
}

impl Default for VirtualMouseBuilder {
    fn default() -> Self {
        Self {
            supports_hi_res: true,
        }
    }
}

impl VirtualMouseBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn supports_hi_res_scroll(&self) -> bool {
        self.supports_hi_res
    }
}

pub struct VirtualMouse {
    _builder: VirtualMouseBuilder,
}

impl VirtualMouse {
    pub fn new(builder: VirtualMouseBuilder) -> Result<Self> {
        Ok(Self { _builder: builder })
    }

    pub fn emit_scroll_hi_res(&mut self, _units: i32) -> Result<()> {
        // High-res scroll dispatch stub for uinput
        Ok(())
    }
}
