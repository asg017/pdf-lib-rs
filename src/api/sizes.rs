/// Standard page sizes as [width, height] in points (1 point = 1/72 inch).
pub struct PageSizes;

#[allow(dead_code)]
impl PageSizes {
    pub const LETTER: [f64; 2] = [612.0, 792.0];
    pub const LEGAL: [f64; 2] = [612.0, 1008.0];
    pub const TABLOID: [f64; 2] = [792.0, 1224.0];
    pub const LEDGER: [f64; 2] = [1224.0, 792.0];
    pub const A0: [f64; 2] = [2383.94, 3370.39];
    pub const A1: [f64; 2] = [1683.78, 2383.94];
    pub const A2: [f64; 2] = [1190.55, 1683.78];
    pub const A3: [f64; 2] = [841.89, 1190.55];
    pub const A4: [f64; 2] = [595.28, 841.89];
    pub const A5: [f64; 2] = [419.53, 595.28];
    pub const A6: [f64; 2] = [297.64, 419.53];
    pub const EXECUTIVE: [f64; 2] = [521.86, 756.0];
    pub const FOLIO: [f64; 2] = [612.0, 936.0];
}
