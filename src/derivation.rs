use std::path::{Path, PathBuf};
use tokio::fs::{ReadDir, read_dir, File};
use tokio::io::{AsyncRead, AsyncReadExt};
use std::str;

pub enum DerivationError {
  IOError(std::io::Error),
  NoMorePhases,
  ExecutorErrored(ExecutorError),
  FileReadError(std::str::Utf8Error),
  YAMLScanError(yaml_rust::ScanError),
  NoOutputPath,
  NoDependencies,
  NoPhases,
  DerivationPhaseMissingName,
  DerivationPhaseMissingExecutor
}

pub struct ExecutorError {
  err: String,
}

impl From<std::io::Error> for DerivationError {
  fn from(err: std::io::Error) -> Self {
    DerivationError::IOError(err)
  }
}

impl From<std::str::Utf8Error> for DerivationError {
  fn from(err: std::str::Utf8Error) -> Self {
    DerivationError::FileReadError(err)
  }
}
impl From<yaml_rust::ScanError> for DerivationError {
  fn from(err: yaml_rust::ScanError) -> Self {
    DerivationError::YAMLScanError(err)
  }
}

pub type Result<T> = std::result::Result<T, DerivationError>;

pub enum InputSourceType {
  Git, Relative, Absolute, Ftp
}


pub type DerivationResult = std::result::Result<Derivation, ExecutorError>;
pub type DerivationExecutor = Box<dyn std::ops::FnMut(&mut Derivation) -> DerivationResult>;

pub struct DerivationPhase {
  name: String,
  executor: DerivationExecutor,
}

impl DerivationPhase {
  pub fn from_yaml(yaml: &Yaml) -> Result<Self> {
    Ok(DerivationPhase {
      name: yaml["name"]
        .into_string()
        .ok_or(DerivationError::DerivationPhaseMissingName)?,
      executor: yaml["executor"]
        .into_string()
        .ok_or(DerivationError::DerivationPhaseMissingExecutor)?
        // TODO: Parse Executor into Executor structure, then convert into FnMut
    })
  }
}

pub struct Derivation {
  output: PathBuf,
  dependencies: Vec<Derivation>,
  phases: Vec<DerivationPhase>
}

use yaml_rust::Yaml;
impl Derivation {
  pub fn do_next_phase(&mut self) -> Result<Derivation> {
    let mut phase = match self.phases.pop() {
      None => return Err(DerivationError::NoMorePhases),
      Some(phase) => phase,
    };

    phase
      .executor
      .call_mut( (self, ) )
      .map_err(|err| DerivationError::ExecutorErrored(err))
  }

  pub async fn load_from_yaml(yaml: &Yaml) -> Result<Derivation> {
    use yaml_rust::YamlLoader;
    let output = Path::new(yaml["output"]
      .as_str()
      .ok_or(DerivationError::NoOutputPath)?)
      .to_path_buf();


    let mut dependencies = Vec::new();
    for yaml in yaml["dependencies"]
      .as_vec()
      .ok_or(DerivationError::NoDependencies)? 
    {
      let derivation = Derivation::load_from_yaml(yaml).await?;
      dependencies.push(derivation);
    }

    let mut phases = Vec::new();
    for yaml in yaml["phases"]
      .as_vec()
      .ok_or(DerivationError::NoPhases)
    {
      let phase = 
    }

    Ok(Derivation {
      output,
      dependencies,
      phases: yaml["phases"]
    })
  }
}