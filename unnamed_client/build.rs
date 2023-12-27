////////////////////////////////////////////////////////////////////////////////
//
//                ██╗   ██╗███╗   ██╗███████╗███╗   ██╗
//                ██║   ██║████╗  ██║██╔════╝████╗  ██║
//                ██║   ██║██╔██╗ ██║█████╗  ██╔██╗ ██║
//                ██║   ██║██║╚██╗██║██╔══╝  ██║╚██╗██║
//                ╚██████╔╝██║ ╚████║███████╗██║ ╚████║
//                 ╚═════╝ ╚═╝  ╚═══╝╚══════╝╚═╝  ╚═══╝ CLIENT
//
////////////////////////////////////////////////////////////////////////////////
// ? Build helper.
extern crate winres;

fn main() {
    if cfg!(target_os = "windows") {
        // Set the icon of the output binary
        let mut res = winres::WindowsResource::new();
        res.set_icon("../resource/branding/unnamed_engine_icon.ico");
        res.compile().unwrap();
    }
}
