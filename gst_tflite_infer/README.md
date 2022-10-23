Requirements
* bazelisk `npm install -g @bazel/bazelisk` for tflitec build
* If compiling on x64, ensure QEMU is installed to avoid exec errors:
  * `docker run --privileged --rm tonistiigi/binfmt --install all`
