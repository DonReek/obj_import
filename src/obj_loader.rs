use std::{collections::HashMap, error::Error, fs::File, io::{BufRead, BufReader}, str::FromStr};
use l_alg::{Vec3, Vec2};
use crate::*;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct FaceIndex{
    pos:i32,
    tex:i32,
    norm:i32
}

type FaceIndices = Vec<FaceIndex>;

#[derive(Debug)]
struct ObjData{
    vert_positions:Vec<Vec3>,
    tex_coords:Vec<Vec2>,
    normals:Vec<Vec3>,
    faces: Vec<FaceIndices>
}

impl FaceIndex{
    pub fn new(pos:i32, tex:i32, norm:i32)->Self{
        FaceIndex { pos, tex, norm}
    }
}

impl ObjData{
    pub fn new(vert_positions:Vec<Vec3>, tex_coords:Vec<Vec2>, normals:Vec<Vec3>, faces:Vec<FaceIndices>)->Self{
        ObjData{vert_positions, tex_coords, normals, faces}
    }
}

#[derive(Debug, PartialEq, Clone)]
struct ObjObject{
    index:usize,
    pos: Vec3,
    tex_coord:Vec2,
    normal:Vec3
}

impl ObjObject{
    pub fn new(index:usize, pos:&Vec3)->Self{
        ObjObject { index, pos:pos.clone(), tex_coord:Vec2::new(0.,0.), normal: Vec3::new(0.,0.,0.)}
    }
}

fn obj_get_data(file_loc:&str)->Result<ObjData, Box<dyn Error>>{
    // BREAK FILE INTO LINE STRINGS
    let file = File::open(file_loc).unwrap();
    let lines = BufReader::new(file).lines();

    // PARSE ANY LINES THAT IS MADE UP OF FLOATS
    let parse_floats = |s:&str| {
        let mut nums:Vec<f64> = Vec::new();
        let num_strs = s.split_whitespace();
        for num_str in num_strs{
            nums.push(f64::from_str(num_str).unwrap());
        }
        nums
    };

    // DATA VECS
    let mut verts:Vec<Vec3> = Vec::new();
    let mut tex_coords:Vec<Vec2> = Vec::new();
    let mut normals:Vec<Vec3> = Vec::new();
    let mut faces:Vec<FaceIndices> = Vec::new(); 

    // DATA GATHERING: Iterate over line strings
    for line in lines{
        let line_str = line.unwrap();
        // POSITIONS
        if line_str.find("v ") == Some(0){
            let line_slc = &line_str[2..];

            let floats = parse_floats(line_slc);
            verts.push(Vec3::new(floats[0], floats[1], floats[2]));
        }
        // TEX COORDS
        else if line_str.find("vt ") == Some(0){
            let line_slc = &line_str[3..];

            let floats = parse_floats(line_slc);
            tex_coords.push(Vec2::new(floats[0], floats[1]));
        }
        // NORMS
        else if line_str.find("vn ") == Some(0){
            let line_slc = &line_str[3..];

            let floats = parse_floats(line_slc);
            normals.push(Vec3::new(floats[0], floats[1], floats[2]));
        }
        // FACE INDICES (pos/tex/norm)
        else if line_str.find("f ") == Some(0){
            let line_slc = &line_str[2..];

            let mut face_indices = FaceIndices::new();
            let str_indices = line_slc.split_whitespace();

            for str_index in str_indices{
                let parts:Vec<&str> = str_index.split("/").collect();
                let mut face_ind = FaceIndex::new(0,0,0);
                if parts.len() > 0 {
                    face_ind.pos = i32::from_str(parts[0]).unwrap()-1;
                }
                if parts.len() > 1 && parts[1] != ""{
                    face_ind.tex = i32::from_str(parts[1]).unwrap()-1;
                }
                if parts.len() > 2 {
                    face_ind.norm = i32::from_str(parts[2]).unwrap()-1;
                }
                face_indices.push(face_ind);
            }
            faces.push(face_indices);
        }
    }

    Ok(ObjData::new(verts,tex_coords,normals,faces))
}

fn index_data(obj_data:ObjData, contains_tex_coords:bool, contains_normals:bool)->(Vec<u32>, Vec<f32>){
    let mut indices:Vec<u32> = Vec::new();
    let mut obj_map: HashMap<FaceIndex, ObjObject> = HashMap::new();
    let mut data_vec:Vec<ObjObject> = Vec::new();

    let mut check_index = |fi:&FaceIndex| {
        let objobj = obj_map.get(fi);
        let obj_index:usize;
        if objobj == None{
            let mut new_obj = ObjObject::new(
                data_vec.len(),
                &obj_data.vert_positions[fi.pos as usize],
            );
            if contains_tex_coords{
                new_obj.tex_coord = obj_data.tex_coords[fi.tex as usize].clone();
            }
            if contains_normals{
                new_obj.normal = obj_data.normals[fi.norm as usize].clone();
            }
            obj_map.insert(fi.clone(), new_obj.clone());
            data_vec.push(new_obj.clone());
            obj_index = new_obj.index;
        }
        else{
            obj_index = objobj.unwrap().index;
        }
        obj_index as u32
    };

    for face in &obj_data.faces{
        let mut face_c = face.clone();

        let first_ind = check_index(&face_c[0]);
        face_c.remove(0);
        while face_c.len() >= 2{
            indices.push(first_ind);
            indices.push(check_index(&face_c[0]));
            indices.push(check_index(&face_c[1]));
            face_c.remove(0);
        }
    }

    let mut raw_data = Vec::new();
    for chunk in &data_vec{
        raw_data.push(chunk.pos.x as f32);
        raw_data.push(chunk.pos.y as f32);
        raw_data.push(chunk.pos.z as f32);
        if contains_tex_coords{
            raw_data.push(chunk.tex_coord.x as f32);
            raw_data.push(chunk.tex_coord.y as f32);
        }
        if contains_normals{
            raw_data.push(chunk.normal.x as f32);
            raw_data.push(chunk.normal.y as f32);
            raw_data.push(chunk.normal.z as f32);
        }
    }

    (indices, raw_data)
} 

impl ObjLoader{
    pub fn from_file(file_loc:&str)->Self{
        let obj_data = obj_get_data(file_loc).unwrap();
        let mut contains_tex_coords = false;
        let mut contains_normals = false;
        if obj_data.tex_coords.len() > 0{
            contains_tex_coords = true;
        }
        if obj_data.normals.len() > 0{
            contains_normals = true;
        }
        let (indices, vert_data ) = index_data(obj_data, contains_tex_coords, contains_normals);
        ObjLoader { indices, vert_data, contains_tex_coords,contains_normals }
    }
    
    pub fn contains_tex_coords(&self)->bool{
        self.contains_tex_coords
    }
    
    pub fn contains_normals(&self)->bool{
        self.contains_normals
    }

    /// Gets the vertex data and indices from the loader. Passes ownership
    /// of the data, consuming the loader in the process. 
    pub fn get_data(self)->(Vec<f32>, Vec<u32>){
        (self.vert_data, self.indices)
    }
}