// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// A `docker-compose.yml` file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct File {
    /// The version of the `docker-compose.yml` file format.  Must be 2.
    #[serde(deserialize_with = "check_version")]
    version: String,
    // TODO HIGH: Remove phantom and make sure `version` defaults correctly.

    /// The individual services which make up this app.
    pub services: BTreeMap<String, Service>,

    /// PRIVATE.  Mark this struct as having unknown fields for future
    /// compatibility.  This prevents direct construction and exhaustive
    /// matching.  This needs to be be public because of
    /// http://stackoverflow.com/q/39277157/12089
    #[doc(hidden)]
    #[serde(default, skip_serializing, skip_deserializing)]
    pub _phantom: PhantomData<()>,
}

derive_merge_override_for!(File, {
    version, services, _phantom
});

impl File {
    /// Read a file from an input stream containing YAML.
    pub fn read<R>(r: R) -> Result<Self, Error>
        where R: io::Read
    {
        Ok(try!(serde_yaml::from_reader(r)))
    }

    /// Write a file to an output stream as YAML.
    pub fn write<W>(&self, w: &mut W) -> Result<(), Error>
        where W: io::Write
    {
        Ok(try!(serde_yaml::to_writer(w, self)))
    }

    /// Read a file from the specified path.
    pub fn read_from_path<P>(path: P) -> Result<Self, Error>
        where P: AsRef<Path>
    {
        Self::read(try!(fs::File::open(path)))
    }

    /// Write a file to the specified path.
    pub fn write_to_path<P>(&self, path: P) -> Result<(), Error>
        where P: AsRef<Path>
    {
        self.write(&mut try!(fs::File::create(path)))
    }
}

impl Default for File {
    fn default() -> File {
        File {
            version: "2".to_owned(),
            services: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl FromStr for File {
    type Err = serde_yaml::Error;

    fn from_str(s: &str) -> Result<File, Self::Err> {
        serde_yaml::from_str(&s)
    }
}

#[test]
fn file_can_be_converted_from_and_to_yaml() {
    let yaml = r#"---
"services":
  "foo":
    "build": "."
"version": "2"
"#;
    assert_roundtrip!(File, yaml);

    let file: File = serde_yaml::from_str(&yaml).unwrap();
    let foo = file.services.get("foo").unwrap();
    assert_eq!(foo.build.as_ref().unwrap().context, value(Context::new(".")));
}

#[test]
fn file_can_only_load_from_version_2() {
    let yaml = r#"---
"services":
  "foo":
    "build": "."
"version": "3"
"#;
    assert!(serde_yaml::from_str::<File>(&yaml).is_err());
}
