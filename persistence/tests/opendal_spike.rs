
#[cfg(test)]
mod test {

    use opendal::layers::LoggingLayer;
    use opendal::services;
    use opendal::Operator;
    use opendal::Result;

    #[tokio::test]
    async fn test_opendal_0() -> Result<()> {

        // Pick a builder and configure it.
        let mut builder = services::Fs::default();
        builder.root("./");

        // Init an operator
        let op = Operator::new(builder)?
            // Init with logging layer enabled.
            .layer(LoggingLayer::default())
            .finish();

        // Write data
        // op.write("hello.txt", "Hello, World!").await?;

        // Read data
        let bs = op.read("../Cargo.toml").await?;
        println!("{}", String::from_utf8_lossy(&bs));

        // Fetch metadata
        // let meta = op.stat("hello.txt").await?;
        // let mode = meta.mode();
        // let length = meta.content_length();

        // // Delete
        // op.delete("hello.txt").await?;

        Ok(())

    }
}