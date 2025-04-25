mod obj_loader;

pub struct ObjLoader{
    pub indices:Vec<u32>,
    pub vert_data:Vec<f32>,
    contains_tex_coords:bool,
    contains_normals:bool
}