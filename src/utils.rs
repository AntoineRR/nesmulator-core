/// A utiliy struct to represent an color.
/// The alpha channel is not calculated by the emulator (set to 255).
#[derive(Debug, Clone, Copy)]
pub struct ARGBColor {
    pub alpha: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl ARGBColor {
    /// Create a new color
    pub const fn new(alpha: u8, red: u8, green: u8, blue: u8) -> Self {
        ARGBColor {
            alpha,
            red,
            green,
            blue,
        }
    }

    /// Return a black color (255, 0, 0, 0)
    pub fn black() -> Self {
        ARGBColor::new(255, 0, 0, 0)
    }

    /// Return a light gray color (255, 50, 50, 50)
    pub fn light_gray() -> Self {
        ARGBColor::new(255, 50, 50, 50)
    }
}

/// A utility enum to represent each input possible on a NES controller.
///
/// # Example
///
/// Using `winit_input_helper` crate:
/// ```
/// use winit::event::VirtualKeyCode;
/// use winit_input_helper::WinitInputHelper;
///
/// use nes_emulator::nes::NES;
/// use nes_emulator::utils::ControllerInput;
///
/// let mut nes = NES::new();
/// let input_helper = WinitInputHelper::new();
///
/// let mut input = 0;
/// if input_helper.key_held(VirtualKeyCode::Z) {
///     input |= ControllerInput::Up as u8;
/// }
/// if input_helper.key_held(VirtualKeyCode::Q) {
///     input |= ControllerInput::Left as u8;
/// }
/// if input_helper.key_held(VirtualKeyCode::S) {
///     input |= ControllerInput::Down as u8;
/// }
/// if input_helper.key_held(VirtualKeyCode::D) {
///     input |= ControllerInput::Right as u8;
/// }
/// if input_helper.key_held(VirtualKeyCode::X) {
///     input |= ControllerInput::Start as u8;
/// }
/// if input_helper.key_held(VirtualKeyCode::C) {
///     input |= ControllerInput::Select as u8;
/// }
/// if input_helper.key_held(VirtualKeyCode::I) {
///     input |= ControllerInput::A as u8;
/// }
/// if input_helper.key_held(VirtualKeyCode::O) {
///     input |= ControllerInput::B as u8;
/// }
/// nes.input(0, input).unwrap();
/// ```
#[derive(Debug)]
pub enum ControllerInput {
    Right = 0b0000_0001,
    Left = 0b0000_0010,
    Down = 0b0000_0100,
    Up = 0b0000_1000,
    Start = 0b0001_0000,
    Select = 0b0010_0000,
    B = 0b0100_0000,
    A = 0b1000_0000,
}
