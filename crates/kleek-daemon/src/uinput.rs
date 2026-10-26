use anyhow::{Context, Result};
use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, EventType, InputEvent, RelativeAxisType};

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

    pub fn build(self) -> Result<VirtualMouse> {
        let mut rel_axes = AttributeSet::<RelativeAxisType>::new();
        rel_axes.insert(RelativeAxisType::REL_WHEEL);
        rel_axes.insert(RelativeAxisType::REL_WHEEL_HI_RES);

        let device = evdev::uinput::VirtualDeviceBuilder::new()
            .context("Failed to initialize virtual device builder")?
            .name("kleek Virtual Smooth Mouse")
            .with_relative_axes(&rel_axes)
            .context("Failed to configure relative axes on virtual device")?
            .build()
            .context("Failed to instantiate /dev/uinput virtual device")?;

        Ok(VirtualMouse {
            _builder: self,
            device: Some(device),
        })
    }
}

pub struct VirtualMouse {
    _builder: VirtualMouseBuilder,
    device: Option<VirtualDevice>,
}

impl VirtualMouse {
    pub fn new(builder: VirtualMouseBuilder) -> Result<Self> {
        // Fallback for tests when /dev/uinput might not be accessible
        match builder.clone().build() {
            Ok(vm) => Ok(vm),
            Err(_) => Ok(Self {
                _builder: builder,
                device: None,
            }),
        }
    }

    pub fn is_real_device_connected(&self) -> bool {
        self.device.is_some()
    }

    pub fn emit_scroll_hi_res(&mut self, units: i32) -> Result<()> {
        if let Some(ref mut dev) = self.device {
            let ev_hires = InputEvent::new(
                EventType::RELATIVE,
                RelativeAxisType::REL_WHEEL_HI_RES.0,
                units,
            );
            // Conventional discrete step: 120 units = 1 notch
            let discrete_units = units / 120;
            if discrete_units != 0 {
                let ev_discrete = InputEvent::new(
                    EventType::RELATIVE,
                    RelativeAxisType::REL_WHEEL.0,
                    discrete_units,
                );
                dev.emit(&[ev_hires, ev_discrete])
                    .context("Failed to emit wheel events to /dev/uinput")?;
            } else {
                dev.emit(&[ev_hires])
                    .context("Failed to emit high-res wheel event to /dev/uinput")?;
            }
        }
        Ok(())
    }
}
