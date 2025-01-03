use std::{array, fmt, io, string};

// Outward-facing error for the user of this lib. Groups all known potential failures of this library into the only two cases the user needs to care about.
pub enum StormworksParserError {
	NotMesh,
	CorruptFile(Box<dyn SpecificError>)
}
impl std::error::Error for StormworksParserError {}


// Error types unique to this lib
pub(crate) struct SubMeshIndexOutOfBounds {pub submesh_id: u32, pub index: u32, pub relevant_bound: u32}
pub(crate) struct IndexIndexOutOfBounds {pub index: u32, pub vertex_count: u32}
pub(crate) struct TooBigNameLength;//previously known as larderous
pub(crate) struct InvalidStormworksShaderType(pub u16);

// SpecificError serves to group all potential errors this function can fail with, and no more.
pub(crate) trait SpecificError: fmt::Display+fmt::Debug + Send + Sync {}
impl SpecificError for string::FromUtf8Error {}
impl SpecificError for io::Error {}
impl SpecificError for array::TryFromSliceError {}
impl SpecificError for SubMeshIndexOutOfBounds {}
impl SpecificError for IndexIndexOutOfBounds {}
impl SpecificError for TooBigNameLength {}
impl SpecificError for InvalidStormworksShaderType {}

// The actual error message for the error types that are unique to this lib
impl fmt::Display for SubMeshIndexOutOfBounds {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		 write!(f, "Submesh {}'s indexbuffer either starts or runs out of bounds: index {} exceeds bound: {}", self.submesh_id, self.index, self.relevant_bound)
	}
}
impl fmt::Display for IndexIndexOutOfBounds {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "While building indices, an index, number {}, exceeded the vertex count, {}", self.index, self.vertex_count)
  }
}
impl fmt::Display for TooBigNameLength {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	  write!(f, "name_length_bytes is extremely larderous")
  }
}
impl fmt::Display for InvalidStormworksShaderType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	  write!(f, "Tried to make shader with type: {}", self.0)
  }
}
// Copied for debug
impl fmt::Debug for SubMeshIndexOutOfBounds {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		 write!(f, "Submesh {}'s indexbuffer either starts or runs out of bounds: index {} exceeds bound: {}", self.submesh_id, self.index, self.relevant_bound)
	}
}
impl fmt::Debug for IndexIndexOutOfBounds {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "While building indices, an index, number {}, exceeded the vertex count, {}", self.index, self.vertex_count)
  }
}
impl fmt::Debug for TooBigNameLength {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	  write!(f, "name_length_bytes is extremely larderous")
  }
}
impl fmt::Debug for InvalidStormworksShaderType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	  write!(f, "Tried to make shader with type: {}", self.0)
  }
}

impl fmt::Display for StormworksParserError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		 match self {
			  StormworksParserError::NotMesh => write!(f, "File is not a .mesh"),
			  StormworksParserError::CorruptFile(err) => {
					write!(f, "File doesn't represent a valid mesh - Did you try to parse a non-stormworks mesh, or is the file corrupted? Internal library error: {}", err)
			  }
		 }
	}
}
impl fmt::Debug for StormworksParserError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			 StormworksParserError::NotMesh => write!(f, "File is not a .mesh"),
			 StormworksParserError::CorruptFile(err) => {
				  write!(f, "File doesn't represent a valid mesh - Did you try to parse a non-stormworks mesh, or is the file corrupted? Internal library error: {}", err)
			 }
		}
  }
}

// Convenience, so ? can be used everywhere.
impl From<string::FromUtf8Error> for Box<dyn SpecificError> {
    fn from(err: string::FromUtf8Error) -> Self {
        Box::new(err)
    }
}
impl From<io::Error> for Box<dyn SpecificError> {
    fn from(err: io::Error) -> Self {
        Box::new(err)
    }
}
impl From<array::TryFromSliceError> for Box<dyn SpecificError> {
    fn from(err: array::TryFromSliceError) -> Self {
        Box::new(err)
    }
}
impl From<SubMeshIndexOutOfBounds> for Box<dyn SpecificError> {
    fn from(err: SubMeshIndexOutOfBounds) -> Self {
        Box::new(err)
    }
}
impl From<IndexIndexOutOfBounds> for Box<dyn SpecificError> {
    fn from(err: IndexIndexOutOfBounds) -> Self {
        Box::new(err)
    }
}
impl From<TooBigNameLength> for Box<dyn SpecificError> {
    fn from(err: TooBigNameLength) -> Self {
        Box::new(err)
    }
}
impl From<InvalidStormworksShaderType> for Box<dyn SpecificError> {
    fn from(err: InvalidStormworksShaderType) -> Self {
        Box::new(err)
    }
}

impl From<Box<dyn SpecificError>> for StormworksParserError {
	fn from(value: Box<dyn SpecificError>) -> Self {
		 StormworksParserError::CorruptFile(value)
	}
}
impl From<string::FromUtf8Error> for StormworksParserError {
	fn from(err: string::FromUtf8Error) -> Self {
		StormworksParserError::CorruptFile(Box::new(err))
	}
}
impl From<io::Error> for StormworksParserError {
	fn from(err: io::Error) -> Self {
		StormworksParserError::CorruptFile(Box::new(err))
	}
}
impl From<array::TryFromSliceError> for StormworksParserError {
	fn from(err: array::TryFromSliceError) -> Self {
		 StormworksParserError::CorruptFile(Box::new(err))
	}
}
impl From<SubMeshIndexOutOfBounds> for StormworksParserError {
	fn from(err: SubMeshIndexOutOfBounds) -> Self {
		 StormworksParserError::CorruptFile(Box::new(err))
	}
}
impl From<IndexIndexOutOfBounds> for StormworksParserError {
	fn from(err: IndexIndexOutOfBounds) -> Self {
		 StormworksParserError::CorruptFile(Box::new(err))
	}
}
impl From<TooBigNameLength> for StormworksParserError {
	fn from(err: TooBigNameLength) -> Self {
		 StormworksParserError::CorruptFile(Box::new(err))
	}
}
impl From<InvalidStormworksShaderType> for StormworksParserError {
	fn from(err: InvalidStormworksShaderType) -> Self {
		 StormworksParserError::CorruptFile(Box::new(err))
	}
}