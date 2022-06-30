pub(crate) trait IntoMeili {
    fn write_into_meili(&self, uri: String, key: String);
}
