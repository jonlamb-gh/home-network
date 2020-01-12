pub mod getset;
pub mod parameter;
pub mod parameter_id_list;
pub mod parameter_list;

pub mod field {
    pub type Field = ::core::ops::Range<usize>;
    pub type Rest = ::core::ops::RangeFrom<usize>;
}
