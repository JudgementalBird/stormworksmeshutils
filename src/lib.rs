use std::{fs::File, io::{BufReader, Read, Seek, SeekFrom}};

use vek::{vec::repr_c::vec3::Vec3, Rgba};

const BYTES_PER_COMPONENT: usize  = 4;
const ENTRIES_PER_VERTEX: usize = 7;
const BYTES_PER_VERTEX: usize = BYTES_PER_COMPONENT*ENTRIES_PER_VERTEX;

pub struct MeshVertexRecord {
    position: Vec3<f32>,
    color: Rgba<u8>,
    normal: Vec3<f32>
}
#[derive(Debug)]
pub enum ShaderType {
    Opaque = 0,
    Transparent = 1,
    Emissive = 2,
    Lava = 3
}
impl ShaderType {
    fn from_u16(i: u16) -> Self {
        match i {
            0 => ShaderType::Opaque,
            1 => ShaderType::Transparent,
            2 => ShaderType::Emissive,
            3 => ShaderType::Lava,
            _ => panic!("Tried to make ShaderType from u16 with value {i}")
        }
    }
}
pub struct SubMesh {
    index_buffer_start: u32,
    index_buffer_length: u32,
    shader_id: ShaderType,
    name_length_bytes: u16,
    name: String,
}
pub struct Mesh {
    vertex_count: u32,
    vertices: Vec<MeshVertexRecord>,
    index_count: u32,
    indices: Vec<u32>,
    sub_mesh_count: u32,
    sub_meshes: Vec<SubMesh>,
}

fn read_u16_from(reader: &mut BufReader<File>) -> u16 {
    let mut byte_buffer: [u8;2] = [0;2];
    reader.read_exact(&mut byte_buffer).unwrap();
    u16::from_le_bytes(byte_buffer)
}
fn read_u32_from(reader: &mut BufReader<File>) -> u32 {
    let mut byte_buffer: [u8;4] = [0;4];
    reader.read_exact(&mut byte_buffer).unwrap();
    u32::from_le_bytes(byte_buffer)
}


// Our version of `public VertexRecord(byte[] bytes)`
fn build_vertex_record(mesh_stream: &mut BufReader<File>) -> MeshVertexRecord {

    let mut vertex_record_bytes = [0_u8;BYTES_PER_VERTEX];
    mesh_stream.read_exact(&mut vertex_record_bytes).unwrap();

    let position_x = f32::from_le_bytes(vertex_record_bytes[0..=3].try_into().unwrap());
    let position_y = f32::from_le_bytes(vertex_record_bytes[4..=7].try_into().unwrap());
    let position_z = f32::from_le_bytes(vertex_record_bytes[8..=11].try_into().unwrap());
    let position = Vec3::new(position_x, position_y, position_z);

    let red = vertex_record_bytes[12];
    let green = vertex_record_bytes[13];
    let blue = vertex_record_bytes[14];
    let alpha = vertex_record_bytes[15];
    let color = Rgba::new(red, green, blue, alpha);

    let normal_x = f32::from_le_bytes(vertex_record_bytes[16..=19].try_into().unwrap());
    let normal_y = f32::from_le_bytes(vertex_record_bytes[20..=23].try_into().unwrap());
    let normal_z = f32::from_le_bytes(vertex_record_bytes[24..=27].try_into().unwrap());
    let normal = Vec3::new(normal_x,normal_y,normal_z);

    MeshVertexRecord {
        position,
        color,
        normal
    } 
}

fn build_vertices(mesh_stream: &mut BufReader<File>, vertex_count: u32) -> Vec<MeshVertexRecord> {
    let mut vertices = Vec::new();
    for _ in 0..vertex_count {
        vertices.push(build_vertex_record(mesh_stream));
    }
    return vertices
}

fn build_indices(mesh_stream: &mut BufReader<File>, index_count: u32, vertex_count: u32) -> Vec<u32> {
    let mut indices = Vec::new();
    for i in 0..index_count {
        let index = read_u16_from(mesh_stream) as u32;
        if index >= vertex_count {
            panic!("Index #{i} is out of bounds {index} > {vertex_count}.");
        }
        indices.push(index);
    }
    return indices
}

fn build_sub_mesh(mesh_stream: &mut BufReader<File>) -> SubMesh {
    let index_buffer_start = read_u32_from(mesh_stream);

    let index_buffer_length = read_u32_from(mesh_stream);

    mesh_stream.seek(SeekFrom::Current(2)).unwrap(); // Header 2

    let shader_id = ShaderType::from_u16(
        read_u16_from(mesh_stream)
    );

    mesh_stream.seek(SeekFrom::Current(4*3+4*3+2)).unwrap();

    let name_length_bytes = read_u16_from(mesh_stream);
    
    if name_length_bytes > 1_000 {
        panic!("name_length_bytes is extremely larderous.")
    }

    let mut name_buf = Vec::with_capacity(name_length_bytes as usize);
    for _ in 0..name_length_bytes {
        name_buf.push(0);
    }
    mesh_stream.read_exact(&mut name_buf).unwrap();
    //println!("debg: {:?}",name_buf);
    let name = String::from_utf8(name_buf).unwrap();

    mesh_stream.seek(SeekFrom::Current(4*3)).unwrap(); // Header 8
    
    SubMesh {
        index_buffer_start,
        index_buffer_length,
        shader_id,
        name_length_bytes,
        name
    }
}

fn build_sub_meshes(mesh_stream: &mut BufReader<File>, sub_mesh_count: u32, index_count: u32) -> Vec<SubMesh> {
    let mut sub_meshes = Vec::with_capacity(sub_mesh_count as usize);
    for i in 0..sub_mesh_count {
        let sub_mesh = build_sub_mesh(mesh_stream);
        
        if sub_mesh.index_buffer_start > index_count {
            panic!("THIS ERROR IS COPIED sub_mesh #{}'s index_buffer starts out of bounds {} > {}",i,sub_mesh.index_buffer_start,index_count);
        }

        if sub_mesh.index_buffer_start + sub_mesh.index_buffer_length > index_count {
            panic!("THIS ERROR IS COPIED sub_mesh #{}'s index_buffer runs out of bounds \n ({} + {} = {}) > {}",i, sub_mesh.index_buffer_start, sub_mesh.index_buffer_length, sub_mesh.index_buffer_start + sub_mesh.index_buffer_length, index_count);
        }
        
        sub_meshes.push(sub_mesh);
    }
    return sub_meshes
}

// our version of `public static Mesh LoadMesh(Stream stream, MeshDiagCallback diag = null)`
pub fn build_mesh(mut mesh_stream: BufReader<File>) -> Result<Mesh,String> {
    // first 4 bytes are 4 chars, the file type header 'mesh'
    let mut filetypemarker: [u8;4] = [0;4];
    mesh_stream.read_exact(&mut filetypemarker).unwrap();
    if filetypemarker != *b"mesh" {
        panic!("not a mesh...")
    }

    // the following 4 bytes are header0 and header1
    mesh_stream.seek(SeekFrom::Current(4)).unwrap();

    let vertex_count = read_u16_from(&mut mesh_stream) as u32;

    // the following 4 bytes are header3 and header4
    mesh_stream.seek(SeekFrom::Current(4)).unwrap();

    let vertices: Vec<MeshVertexRecord> = build_vertices(&mut mesh_stream, vertex_count);

    // on to indices
    let index_count = read_u32_from(&mut mesh_stream);

    let indices = build_indices(&mut mesh_stream, index_count, vertex_count);

    // on to submeshes
    let sub_mesh_count = read_u16_from(&mut mesh_stream) as u32;
    //if sub_mesh_count > 1 {
    //    return Result::Err(String::from("Shit!"));
    //}
    let sub_meshes = build_sub_meshes(&mut mesh_stream, sub_mesh_count, index_count);

    // end of data

    Result::Ok(Mesh {
        vertex_count, // the following 2 bytes are vertex_count
        vertices, // the following 
        index_count,
        indices,
        sub_mesh_count,
        sub_meshes
    })
}

//fn save_ply(mesh: Mesh, ply_stream: BufWriter<File>) {
//    
//}