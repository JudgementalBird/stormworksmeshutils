#![allow(private_interfaces)]

use std::{fs::File, io::{self, BufReader, Read, Seek, SeekFrom}};
use bevy_asset::{io::{AsyncSeekForwardExt, Reader}, AsyncReadExt};
use vek::{vec::repr_c::vec3::Vec3, Rgba};

mod errors;
use errors::*;

const BYTES_PER_COMPONENT: usize  = 4;
const ENTRIES_PER_VERTEX: usize = 7;
const BYTES_PER_VERTEX: usize = BYTES_PER_COMPONENT*ENTRIES_PER_VERTEX;

pub struct StormworksMeshVertexRecord {
    pub position: Vec3<f32>,
    pub color: Rgba<u8>,
    pub normal: Vec3<f32>
}

pub enum StormworksShaderType {
    Opaque = 0,
    Transparent = 1,
    Emissive = 2,
    Lava = 3
}
impl StormworksShaderType {
    fn from_u16(i: u16) -> Result<Self,InvalidStormworksShaderType> {
        match i {
            0 => Ok(StormworksShaderType::Opaque),
            1 => Ok(StormworksShaderType::Transparent),
            2 => Ok(StormworksShaderType::Emissive),
            3 => Ok(StormworksShaderType::Lava),
            _ => Err(InvalidStormworksShaderType(i)),
        }
    }
}
pub struct StormworksSubMesh {
    pub index_buffer_start: u32,
    pub index_buffer_length: u32,
    pub shader_id: StormworksShaderType,
    pub name_length_bytes: u16,
    pub name: String,
}
pub struct StormworksMesh {
    pub vertex_count: u32,
    pub vertices: Vec<StormworksMeshVertexRecord>,
    pub index_count: u32,
    pub indices: Vec<u32>,
    pub sub_mesh_count: u32,
    pub sub_meshes: Vec<StormworksSubMesh>,
}

fn read_u16_from(reader: &mut BufReader<File>) -> Result<u16,io::Error> {
    let mut byte_buffer: [u8;2] = [0;2];
    reader.read_exact(&mut byte_buffer)?;
    Ok(u16::from_le_bytes(byte_buffer))
}
fn read_u32_from(reader: &mut BufReader<File>) -> Result<u32,io::Error> {
    let mut byte_buffer: [u8;4] = [0;4];
    reader.read_exact(&mut byte_buffer)?;
    Ok(u32::from_le_bytes(byte_buffer))
}


// Our version of `public VertexRecord(byte[] bytes)`
fn build_vertex_record(mesh_stream: &mut BufReader<File>) -> Result<StormworksMeshVertexRecord,Box<dyn SpecificError>> {

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

    Ok(StormworksMeshVertexRecord {
        position,
        color,
        normal
    })
}

fn build_vertices(mesh_stream: &mut BufReader<File>, vertex_count: u32) -> Result<Vec<StormworksMeshVertexRecord>,Box<dyn SpecificError>> {
    let mut vertices = Vec::new();
    for _ in 0..vertex_count {
        vertices.push(build_vertex_record(mesh_stream)?);
    }
    return Ok(vertices)
}

fn build_indices(mesh_stream: &mut BufReader<File>, index_count: u32, vertex_count: u32) -> Result<Vec<u32>,Box<dyn SpecificError>> {
    let mut indices = Vec::new();
    for i in 0..index_count {
        let index = read_u16_from(mesh_stream)? as u32;
        if index >= vertex_count {
            return Err(IndexIndexOutOfBounds { index: i, vertex_count }.into());
        }
        indices.push(index);
    }
    return Ok(indices)
}

fn build_sub_mesh(mesh_stream: &mut BufReader<File>) -> Result<StormworksSubMesh,Box<dyn SpecificError>> {
    let index_buffer_start = read_u32_from(mesh_stream)?;

    let index_buffer_length = read_u32_from(mesh_stream)?;

    mesh_stream.seek(SeekFrom::Current(2))?; // Header 2

    let shader_id = StormworksShaderType::from_u16(
        read_u16_from(mesh_stream)?
    )?;

    mesh_stream.seek(SeekFrom::Current(4*3+4*3+2))?;

    let name_length_bytes = read_u16_from(mesh_stream)?;
    
    if name_length_bytes > 1_000 {
        return Err(TooBigNameLength.into());
    }

    let mut name_buf = Vec::with_capacity(name_length_bytes as usize);
    for _ in 0..name_length_bytes {
        name_buf.push(0);
    }
    mesh_stream.read_exact(&mut name_buf)?;
    
    let name = String::from_utf8(name_buf)?;

    mesh_stream.seek(SeekFrom::Current(4*3))?; // Header 8
    
    Ok(StormworksSubMesh {
        index_buffer_start,
        index_buffer_length,
        shader_id,
        name_length_bytes,
        name
    })
}

fn build_sub_meshes(mesh_stream: &mut BufReader<File>, sub_mesh_count: u32, index_count: u32) -> Result<Vec<StormworksSubMesh>,Box<dyn SpecificError>> {
    let mut sub_meshes = Vec::with_capacity(sub_mesh_count as usize);
    for i in 0..sub_mesh_count {
        let sub_mesh = build_sub_mesh(mesh_stream)?;
        
        if sub_mesh.index_buffer_start > index_count {
            return Err(SubMeshIndexOutOfBounds { submesh_id: i, index: sub_mesh.index_buffer_start, relevant_bound: index_count }.into())
        }

        if sub_mesh.index_buffer_start + sub_mesh.index_buffer_length > index_count {
            return Err(SubMeshIndexOutOfBounds { submesh_id: i, index: sub_mesh.index_buffer_start + sub_mesh.index_buffer_length, relevant_bound: index_count }.into())
        }
        
        sub_meshes.push(sub_mesh);
    }
    return Ok(sub_meshes)
}

// our version of `public static Mesh LoadMesh(Stream stream, MeshDiagCallback diag = null)`
/// yum,,!
pub fn build_stormworks_mesh(mut mesh_stream: BufReader<File>) -> Result<StormworksMesh,StormworksParserError> {
    // first 4 bytes are 4 chars, the file type header 'mesh'
    let mut filetypemarker: [u8;4] = [0;4];
    mesh_stream.read_exact(&mut filetypemarker)?;
    if filetypemarker != *b"mesh" {
        return Err(StormworksParserError::NotMesh);
        
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

    Result::Ok(StormworksMesh {
        vertex_count, // the following 2 bytes are vertex_count
        vertices, // the following 
        index_count,
        indices,
        sub_mesh_count,
        sub_meshes
    })
}

async fn async_read_u16_from(reader: &mut Box<dyn Reader>) -> Result<u16,io::Error> {
    let mut byte_buffer: [u8;2] = [0;2];
    reader.read_exact(&mut byte_buffer).await?;
    Ok(u16::from_le_bytes(byte_buffer))
}
async fn async_read_u32_from(reader: &mut Box<dyn Reader>) -> Result<u32,io::Error> {
    let mut byte_buffer: [u8;4] = [0;4];
    reader.read_exact(&mut byte_buffer).await?;
    Ok(u32::from_le_bytes(byte_buffer))
}


// Our version of `public VertexRecord(byte[] bytes)`
async fn async_build_vertex_record(mesh_stream: &mut Box<dyn Reader>) -> Result<StormworksMeshVertexRecord,Box<dyn SpecificError>> {

    let mut vertex_record_bytes = [0_u8;BYTES_PER_VERTEX];
    mesh_stream.read_exact(&mut vertex_record_bytes).await?;

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

    Ok(StormworksMeshVertexRecord {
        position,
        color,
        normal
    })
}

async fn async_build_vertices(mesh_stream: &mut Box<dyn Reader>, vertex_count: u32) -> Result<Vec<StormworksMeshVertexRecord>,Box<dyn SpecificError>> {
    let mut vertices = Vec::new();
    for _ in 0..vertex_count {
        vertices.push(async_build_vertex_record(mesh_stream).await?);
    }
    return Ok(vertices)
}

async fn async_build_indices(mesh_stream: &mut Box<dyn Reader>, index_count: u32, vertex_count: u32) -> Result<Vec<u32>,Box<dyn SpecificError>> {
    let mut indices = Vec::new();
    for i in 0..index_count {
        let index = async_read_u16_from(mesh_stream).await? as u32;
        if index >= vertex_count {
            return Err(IndexIndexOutOfBounds { index: i, vertex_count }.into());
        }
        indices.push(index);
    }
    return Ok(indices)
}

async fn async_build_sub_mesh(mesh_stream: &mut Box<dyn Reader>) -> Result<StormworksSubMesh,Box<dyn SpecificError>> {
    let index_buffer_start = async_read_u32_from(mesh_stream).await?;

    let index_buffer_length = async_read_u32_from(mesh_stream).await?;

    mesh_stream.seek_forward(2).await?; // Header 2

    let shader_id = StormworksShaderType::from_u16(
        async_read_u16_from(mesh_stream).await?
    )?;

    mesh_stream.seek_forward(4*3+4*3+2).await?;

    let name_length_bytes = async_read_u16_from(mesh_stream).await?;
    
    if name_length_bytes > 1_000 {
        return Err(TooBigNameLength.into());
    }

    let mut name_buf = Vec::with_capacity(name_length_bytes as usize);
    for _ in 0..name_length_bytes {
        name_buf.push(0);
    }
    mesh_stream.read_exact(&mut name_buf).await?;
    
    let name = String::from_utf8(name_buf)?;

    mesh_stream.seek_forward(4*3).await?; // Header 8
    
    Ok(StormworksSubMesh {
        index_buffer_start,
        index_buffer_length,
        shader_id,
        name_length_bytes,
        name
    })
}

async fn async_build_sub_meshes(mesh_stream: &mut Box<dyn Reader>, sub_mesh_count: u32, index_count: u32) -> Result<Vec<StormworksSubMesh>,Box<dyn SpecificError>> {
    let mut sub_meshes = Vec::with_capacity(sub_mesh_count as usize);
    for i in 0..sub_mesh_count {
        let sub_mesh = async_build_sub_mesh(mesh_stream).await?;
        
        if sub_mesh.index_buffer_start > index_count {
            return Err(SubMeshIndexOutOfBounds { submesh_id: i, index: sub_mesh.index_buffer_start, relevant_bound: index_count }.into())
        }

        if sub_mesh.index_buffer_start + sub_mesh.index_buffer_length > index_count {
            return Err(SubMeshIndexOutOfBounds { submesh_id: i, index: sub_mesh.index_buffer_start + sub_mesh.index_buffer_length, relevant_bound: index_count }.into())
        }
        
        sub_meshes.push(sub_mesh);
    }
    return Ok(sub_meshes)
}

// our version of `public static Mesh LoadMesh(Stream stream, MeshDiagCallback diag = null)`
/// yum,,!
pub async fn async_build_stormworks_mesh(mut mesh_stream: Box<dyn Reader>) -> Result<StormworksMesh,StormworksParserError> {

    // first 4 bytes are 4 chars, the file type header 'mesh'
    let mut filetypemarker: [u8;4] = [0;4];
    mesh_stream.read_exact(&mut filetypemarker).await?;
    if filetypemarker != *b"mesh" {
        return Err(StormworksParserError::NotMesh);
        
    }

    // the following 4 bytes are header0 and header1
    mesh_stream.seek_forward(4).await?;

    let vertex_count = async_read_u16_from(&mut mesh_stream).await? as u32;

    // the following 4 bytes are header3 and header4
    mesh_stream.seek_forward(4).await?;

    let vertices = async_build_vertices(&mut mesh_stream, vertex_count).await?;

    // on to indices
    let index_count = async_read_u32_from(&mut mesh_stream).await?;

    let indices = async_build_indices(&mut mesh_stream, index_count, vertex_count).await?;

    // on to submeshes
    let sub_mesh_count = async_read_u16_from(&mut mesh_stream).await? as u32;
    
    let sub_meshes = async_build_sub_meshes(&mut mesh_stream, sub_mesh_count, index_count).await?;

    // end of data

    Result::Ok(StormworksMesh {
        vertex_count, // the following 2 bytes are vertex_count
        vertices, // the following 
        index_count,
        indices,
        sub_mesh_count,
        sub_meshes
    })
}
#[cfg(feature = "bevy-integration")]
impl From<StormworksMesh> for bevy_mesh::Mesh {
    fn from(stormworks_mesh: StormworksMesh) -> Self {
        use bevy_render::render_asset::RenderAssetUsages;

        const LINEAR_RGB_CONVERSION_CONSTANT: f32 = 1.0/2.2;

        let mut verticies = Vec::with_capacity(stormworks_mesh.index_count as usize);
        let mut normals = Vec::with_capacity(stormworks_mesh.index_count as usize);
        let mut colors = Vec::with_capacity(stormworks_mesh.index_count as usize);

        for vertex in &stormworks_mesh.vertices {
            verticies.push([-vertex.position.x,vertex.position.y,vertex.position.z]);
            normals.push([vertex.normal.x,vertex.normal.y,vertex.normal.z]);
            colors.push([
                (vertex.color.r as f32).powf(LINEAR_RGB_CONVERSION_CONSTANT),
                (vertex.color.g as f32).powf(LINEAR_RGB_CONVERSION_CONSTANT),
                (vertex.color.b as f32).powf(LINEAR_RGB_CONVERSION_CONSTANT),
                vertex.color.a as f32
            ]);
        }
        bevy_mesh::Mesh::new(bevy_mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(bevy_mesh::Mesh::ATTRIBUTE_POSITION,verticies)
        .with_inserted_attribute(bevy_mesh::Mesh::ATTRIBUTE_COLOR,colors)
        .with_inserted_attribute(bevy_mesh::Mesh::ATTRIBUTE_NORMAL,normals)
        .with_inserted_indices(bevy_mesh::Indices::U32(stormworks_mesh.indices.clone())) //ask judge about
    }
}