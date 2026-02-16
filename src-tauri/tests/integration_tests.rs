mod fixtures;
mod integration;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fixture_creation() {
        let fixture = fixtures::TestFixture::new();
        assert!(fixture.temp_path().exists());
    }
}
