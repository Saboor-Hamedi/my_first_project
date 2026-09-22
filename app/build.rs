fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "MindForge");
        res.set("FileDescription", "MindForge Note Editor");
        res.set("CompanyName", "MindForge");
        let _ = res.compile();
    }
}
