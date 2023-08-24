use containerd_shim as shim;
use containerd_shim_wasm::sandbox::ShimCli;
use containerd_shim_wasmedge::instance::Wasi as WasiInstance;
use containerd_shim_wasmedge::parse_version;

fn main() {
    parse_version();
    shim::run::<ShimCli<WasiInstance>>("io.containerd.wasmedge.v1", None);
}
