mod fields;

pub struct Opts {
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
    pub usage_page: Option<u16>,
    pub usage: Option<u16>,
}

pub fn get_matching_devices(opts: Opts) -> Vec<fields::Device> {
    vec![]
}