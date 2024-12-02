use std::{array::TryFromSliceError, fs::File, io::{self, BufReader, Read, Seek, SeekFrom}, string::FromUtf8Error};
use vek::{vec::repr_c::vec3::Vec3, Rgba};

const BYTES_PER_COMPONENT: usize  = 4;
const ENTRIES_PER_VERTEX: usize = 7;
const BYTES_PER_VERTEX: usize = BYTES_PER_COMPONENT*ENTRIES_PER_VERTEX;

pub struct MeshVertexRecord {
    pub position: Vec3<f32>,
    pub color: Rgba<u8>,
    pub normal: Vec3<f32>
}

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
    pub index_buffer_start: u32,
    pub index_buffer_length: u32,
    pub shader_id: ShaderType,
    pub name_length_bytes: u16,
    pub name: String,
}
pub struct Mesh {
    pub vertex_count: u32,
    pub vertices: Vec<MeshVertexRecord>,
    pub index_count: u32,
    pub indices: Vec<u32>,
    pub sub_mesh_count: u32,
    pub sub_meshes: Vec<SubMesh>,
}

pub enum Error {
    FromUtf8(FromUtf8Error),
    Io(io::Error),
    FromSlice(TryFromSliceError),
}
impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Error::FromUtf8(value)
    }
}
impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}
impl From<TryFromSliceError> for Error {
    fn from(value: TryFromSliceError) -> Self {
        Error::FromSlice(value)
    }
}

fn read_u16_from(reader: &mut BufReader<File>) -> Result<u16,Error> {
    let mut byte_buffer: [u8;2] = [0;2];
    reader.read_exact(&mut byte_buffer)?;
    Ok(u16::from_le_bytes(byte_buffer))
}
fn read_u32_from(reader: &mut BufReader<File>) -> Result<u32,Error> {
    let mut byte_buffer: [u8;4] = [0;4];
    reader.read_exact(&mut byte_buffer)?;
    Ok(u32::from_le_bytes(byte_buffer))
}


// Our version of `public VertexRecord(byte[] bytes)`
fn build_vertex_record(mesh_stream: &mut BufReader<File>) -> Result<MeshVertexRecord,Error> {

    let mut vertex_record_bytes = [0_u8;BYTES_PER_VERTEX];
    mesh_stream.read_exact(&mut vertex_record_bytes)?;

    let position_x = f32::from_le_bytes(vertex_record_bytes[0..=3].try_into()?);
    let position_y = f32::from_le_bytes(vertex_record_bytes[4..=7].try_into()?);
    let position_z = f32::from_le_bytes(vertex_record_bytes[8..=11].try_into()?);
    let position = Vec3::new(position_x, position_y, position_z);

    let red = vertex_record_bytes[12];
    let green = vertex_record_bytes[13];
    let blue = vertex_record_bytes[14];
    let alpha = vertex_record_bytes[15];
    let color = Rgba::new(red, green, blue, alpha);

    let normal_x = f32::from_le_bytes(vertex_record_bytes[16..=19].try_into()?);
    let normal_y = f32::from_le_bytes(vertex_record_bytes[20..=23].try_into()?);
    let normal_z = f32::from_le_bytes(vertex_record_bytes[24..=27].try_into()?);
    let normal = Vec3::new(normal_x,normal_y,normal_z);

    Ok(MeshVertexRecord {
        position,
        color,
        normal
    })
}

fn build_vertices(mesh_stream: &mut BufReader<File>, vertex_count: u32) -> Result<Vec<MeshVertexRecord>,Error> {
    let mut vertices = Vec::new();
    for _ in 0..vertex_count {
        vertices.push(build_vertex_record(mesh_stream)?);
    }
    return Ok(vertices)
}

fn build_indices(mesh_stream: &mut BufReader<File>, index_count: u32, vertex_count: u32) -> Result<Vec<u32>,Error> {
    let mut indices = Vec::new();
    for i in 0..index_count {
        let index = read_u16_from(mesh_stream)? as u32;
        if index >= vertex_count {
            panic!("Index #{i} is out of bounds {index} > {vertex_count}.");
        }
        indices.push(index);
    }
    return Ok(indices)
}

fn build_sub_mesh(mesh_stream: &mut BufReader<File>) -> Result<SubMesh,Error> {
    let index_buffer_start = read_u32_from(mesh_stream)?;

    let index_buffer_length = read_u32_from(mesh_stream)?;

    mesh_stream.seek(SeekFrom::Current(2))?; // Header 2

    let shader_id = ShaderType::from_u16(
        read_u16_from(mesh_stream)?
    );

    mesh_stream.seek(SeekFrom::Current(4*3+4*3+2))?;

    let name_length_bytes = read_u16_from(mesh_stream)?;
    
    if name_length_bytes > 1_000 {
        panic!("name_length_bytes is extremely larderous.")
    }

    let mut name_buf = Vec::with_capacity(name_length_bytes as usize);
    for _ in 0..name_length_bytes {
        name_buf.push(0);
    }
    mesh_stream.read_exact(&mut name_buf)?;
    
    let name = String::from_utf8(name_buf)?;

    mesh_stream.seek(SeekFrom::Current(4*3))?; // Header 8
    
    Ok(SubMesh {
        index_buffer_start,
        index_buffer_length,
        shader_id,
        name_length_bytes,
        name
    })
}

fn build_sub_meshes(mesh_stream: &mut BufReader<File>, sub_mesh_count: u32, index_count: u32) -> Result<Vec<SubMesh>,Error> {
    let mut sub_meshes = Vec::with_capacity(sub_mesh_count as usize);
    for i in 0..sub_mesh_count {
        let sub_mesh = build_sub_mesh(mesh_stream)?;
        
        if sub_mesh.index_buffer_start > index_count {
            panic!("THIS ERROR IS COPIED sub_mesh #{}'s index_buffer starts out of bounds {} > {}",i,sub_mesh.index_buffer_start,index_count);
        }

        if sub_mesh.index_buffer_start + sub_mesh.index_buffer_length > index_count {
            panic!("THIS ERROR IS COPIED sub_mesh #{}'s index_buffer runs out of bounds \n ({} + {} = {}) > {}",i, sub_mesh.index_buffer_start, sub_mesh.index_buffer_length, sub_mesh.index_buffer_start + sub_mesh.index_buffer_length, index_count);
        }
        
        sub_meshes.push(sub_mesh);
    }
    return Ok(sub_meshes)
}

// our version of `public static Mesh LoadMesh(Stream stream, MeshDiagCallback diag = null)`
/// yum,,!
pub fn build_mesh(mut mesh_stream: BufReader<File>) -> Result<Mesh,Error> {
    // first 4 bytes are 4 chars, the file type header 'mesh'
    let mut filetypemarker: [u8;4] = [0;4];
    mesh_stream.read_exact(&mut filetypemarker)?;
    if filetypemarker != *b"mesh" {
        panic!("not a mesh...")
    }

    // the following 4 bytes are header0 and header1
    mesh_stream.seek(SeekFrom::Current(4))?;

    let vertex_count = read_u16_from(&mut mesh_stream)? as u32;

    // the following 4 bytes are header3 and header4
    mesh_stream.seek(SeekFrom::Current(4))?;

    let vertices = build_vertices(&mut mesh_stream, vertex_count)?;

    // on to indices
    let index_count = read_u32_from(&mut mesh_stream)?;

    let indices = build_indices(&mut mesh_stream, index_count, vertex_count)?;

    // on to submeshes
    let sub_mesh_count = read_u16_from(&mut mesh_stream)? as u32;
    
    let sub_meshes = build_sub_meshes(&mut mesh_stream, sub_mesh_count, index_count)?;

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