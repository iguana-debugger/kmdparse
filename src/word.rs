#[derive(Clone, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum Word {
    /// An instruction, represented as 4 bytes. kmdparse handles flipping the bytes, so that
    /// instructions are the right way around.
    Instruction {
        instruction: [u8; 4],
    },
    Data {
        data: Vec<u8>,
    },
}
