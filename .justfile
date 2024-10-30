# check formatting and clippy
check:
  cargo check
  cargo fmt --all -- --check
  cargo clippy --all-targets --all-features --all -- --deny warnings
  cargo clippy -- --deny warnings

# automatically fromat and fix simple clippy warnings
fix:
  cargo fmt --all
  cargo clippy --allow-dirty --allow-staged --fix

# clean project
clean:
  cargo clean

# build project
build:
  cargo build --all-targets

# build project with release profile
release:
  cargo build --all-targets --release

run-aeron-archive:
    cd ./rusteron-client/aeron; ./gradlew :aeron-all:build; cd -
    java -cp ./rusteron-client/aeron/aeron-all/build/libs/aeron-all-*.jar \
      -Daeron.dir=target/aeron/shm \
      -Darchive.dir=target/aeron/archive \
      -Daeron.term.buffer.sparse.file=false \
      -Daeron.pre.touch.mapped.memory=true \
      -Daeron.socket.so_sndbuf=2m \
      -Daeron.socket.so_rcvbuf=2m \
      -Daeron.rcv.initial.window.length=2m \
      -Daeron.threading.mode=DEDICATED \
      -Daeron.sender.idle.strategy=noop \
      -Daeron.receiver.idle.strategy=noop \
      -Daeron.conductor.idle.strategy=spin \
      -Dagrona.disable.bounds.checks=true \
      -Daeron.archive.control.channel=aeron:udp?endpoint=localhost:8010 \
      -Daeron.archive.replication.channel=aeron:udp?endpoint=localhost:0 \
      -Daeron.archive.control.response.channel=aeron:udp?endpoint=localhost:0 \
      io.aeron.archive.ArchivingMediaDriver

docs:
  cargo clean --doc
  cargo test --doc
  cargo doc --workspace --no-deps --open

# run unit tests
test:
  cargo test --all-targets --all-features -- --nocapture