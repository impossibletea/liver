export LIVE2D_CUBISM_SDK_NATIVE_DIR = ${CURDIR}/res/CubismSdkForNative-4-r.7/

release:
	cargo run --release

run:
	cargo run

doc:
	cargo doc --open
