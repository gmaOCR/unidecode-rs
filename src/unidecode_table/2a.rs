use phf::phf_map;

pub static BLOCK_2A: phf::Map<u32, &'static str> = phf_map!{
    10868u32 => "::=",
    10869u32 => "==",
    10870u32 => "===",

};
