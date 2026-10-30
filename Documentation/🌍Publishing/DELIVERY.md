# LIB.BENCH delivery

LIB.BENCH is not shipped on its own. It is compiled into the binaries that depend on it.

- **Repository:** https://github.com/LikeDotAudio/APK.lib.BENCH (private), a submodule of the
  APK.audio monorepo at `PODS/POD.THICK/LIBRARY/LIB.BENCH`.
- **Build:** as a member of the Cargo workspace at `PODS/POD.THICK/DISCOVERY.Engine/SRC`
  (`Cargo.toml` lists `../../LIBRARY/LIB.BENCH/SRC`). It does not build outside that checkout.
- **Images:** the build stages that compile the workspace copy `PODS/POD.THICK/LIBRARY/LIB.BENCH/SRC`
  into the image: `PODS/POD.THICK/NODE.BareMetal/Docker/Dockerfile` and
  `PODS/POD.databus/DATABUS.Broker.MQTT/Docker/Dockerfile`. Only `SRC/` is copied; the root
  `.dockerignore` excludes every `.git`.
- **Where it lands:** inside the VISA plugin (`apk-visa`) and the orchestrator's VISA hunter
  binaries. No container, port, volume or config file of its own.
- **After an edit:** rebuild the images above (`up-stack.sh` for the stacks that carry them), then
  commit in this repository and bump the submodule pointer in APK.audio.
