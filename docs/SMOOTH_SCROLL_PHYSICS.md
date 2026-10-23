# KLEEK: Smooth Scroll Physics Specification

This document specifies the mathematics and timing architecture for the physics-based smooth scrolling engine in `kleek-daemon`, adapted from the proven principles in **Mos** (`Caldis/Mos`).

---

## 1. Problem Formulation

Traditional mechanical mouse wheels emit discrete notches:
* In Linux `evdev`, one notch generates:
  * `REL_WHEEL = 1` or `-1`
  * `REL_WHEEL_HI_RES = 120` or `-120`
* When dispatched directly to applications, this causes sudden, jarring jumps of 3–5 lines of text at a time.
* Trackpads, by contrast, send hundreds of high-frequency sub-pixel micro-events with smooth momentum decay.

`kleek-daemon` bridges this gap by intercepting discrete notches and translating them into a decaying velocity curve dispatched across high-resolution timer ticks.

---

## 2. Mathematical Model

### 2.1 Velocity Injection (Impulse)
When a physical notch event is detected at time $t_0$, an impulse is added to the current accumulator velocity $v$:

$$v_{\text{new}} = v_{\text{current}} + \text{direction} \times \text{step\_size} \times \text{gain}$$

* `step_size`: Base distance in pixels per notch (default: 40.0 px).
* `gain`: Velocity multiplier based on flick speed (accelerator: if successive notches arrive within $< 50\text{ms}$, gain scales exponentially up to $2.5\times$).

### 2.2 Exponential Decay (Friction Curve)
The velocity decays over time according to an exponential friction function:

$$v(t) = v_0 \cdot e^{-\lambda t}$$

Where:
* $\lambda$ is the damping coefficient (derived from user duration preference: $\lambda = \frac{4.0}{\text{duration}}$).
* Recommended default duration: $0.25\text{s} \implies \lambda = 16.0$.

At each animation tick interval $\Delta t$:

$$v_{k+1} = v_k \cdot (1 - \lambda \cdot \Delta t)$$

When $|v_k| < \epsilon$ (cutoff threshold: 0.5 px/s), the simulation halts and velocity resets to 0.0 to prevent infinite tail calculation.

---

## 3. High-Resolution Event Dispatching (`uinput`)

Linux modern Wayland compositors (Hyprland, GNOME, Sway) and X11 support high-resolution scrolling via `REL_WHEEL_HI_RES` (120 units = 1 notch).

### Tick Cadence
* The loop runs using `tokio::time::interval` at **120Hz** ($\Delta t = 8.33\text{ms}$) or **144Hz** ($\Delta t = 6.94\text{ms}$).
* On each tick:
  $$\Delta \text{distance} = v_k \cdot \Delta t$$
* Fractional distance is accumulated:
  $$\text{hi\_res\_units} = \Delta \text{distance} \times \frac{120}{\text{step\_size}}$$
* The daemon emits `REL_WHEEL_HI_RES` with the integer portion and retains the remainder in a float accumulator for the next tick.

```text
Physical Wheel Click
      │
      ▼
evdev::grab() [Drop raw event from host]
      │
      ▼
Inject Impulse into Physics Accumulator
      │
      ▼ [Timer Tick @ 120Hz]
Calculate v(t) Decay
      │
      ▼
Emit REL_WHEEL_HI_RES via /dev/uinput
```

---

## 4. Invariants & Safety Constraints

1. **Grab Isolation:**  
   When smooth scrolling is active, only `REL_WHEEL` and `REL_WHEEL_HI_RES` are grabbed and consumed by `kleek`. Button clicks and pointer motion (`REL_X`/`REL_Y`) MUST NOT be delayed or re-routed through the timer loop.
2. **Reverse Direction:**  
   Reverse scroll (natural vs traditional) is a simple sign inversion before velocity injection.
3. **App Exclusions (Future-Ready):**  
   If an application requires discrete steps (e.g. game weapon switching, terminal scrolling), the physics accumulator bypasses decay and instantly flushes the full notch.
