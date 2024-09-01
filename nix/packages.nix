{
  self,
  inputs,
  system,
  ...
}: {
  perSystem = {
    inputs',
    self',
    lib,
    pkgs,
    system,
    ...
  }: let

    # These need to be specified because cardano.nix provides multiples
    kupoVersion = "2.9.0";
    ogmiosVersion = "6.5.0";

    flake-compat = import inputs.flake-compat;
    cardanoPackages = (flake-compat { src = inputs.cardano-node; }).defaultNix.packages.${system};
    dbSyncPackages = (flake-compat { src = inputs.cardano-dbsync; }).defaultNix.packages.${system};
    smartContractsPkgs = (flake-compat { src = inputs.smart-contracts; }).defaultNix.packages.${system};
    cardanoExtraPkgs = (flake-compat { src = inputs.cardano-nix; }).defaultNix.packages.${system};

  in {
    packages = rec {
      inherit (smartContractsPkgs) sidechain-main-cli;
      inherit (cardanoPackages) cardano-node cardano-cli cardano-testnet;
      inherit (dbSyncPackages) "cardano-db-sync:exe:cardano-db-sync";
      kupo = cardanoExtraPkgs."kupo-${kupoVersion}";
      ogmios = cardanoExtraPkgs."ogmios-${ogmiosVersion}";
      partnerchains-stack = pkgs.stdenv.mkDerivation {
        name = "partnerchains-stack";
        phases = [ "installPhase" ];
        nativeBuildInputs = [ pkgs.makeWrapper ];
        installPhase = ''
          mkdir -p $out/bin
          cp ${self'.packages.partnerchains-stack-unwrapped}/bin/partnerchains-stack-unwrapped \
            $out/bin/partnerchains-stack
          wrapProgram $out/bin/partnerchains-stack \
            --run "cd \$(${pkgs.git}/bin/git rev-parse --show-toplevel) || exit 1"
        '';
      };
      rustToolchain = let
        fenixPkgs = inputs'.fenix.packages;
        rustToolchain = with fenixPkgs;
          fromToolchainFile {
            file = ../rust-toolchain.toml;
            sha256 = "+syqAd2kX8KVa8/U2gz3blIQTTsYYt3U63xBWaGOSc8=";
          };
        in rustToolchain;
      pc-node = let
        cargo = "${self'.packages.rustToolchain}/bin/cargo";
        customBuildRustCrateForPkgs = pkgs: pkgs.buildRustCrate.override {
          cargo = self'.packages.rustToolchain;
          rustc = self'.packages.rustToolchain;
          defaultCrateOverrides = pkgs.defaultCrateOverrides // rec {
            # WIP, broken still
            openssl-sys = attrs: {
              RUST_BACKTRACE="full";
              PKG_CONFIG_PATH = "${pkgs.pkg-config}";
              nativeBuildInputs = [
                pkgs.pkg-config
                pkgs.breakpointHook
              ];
              CARGO = cargo;
              buildInputs = [
                pkgs.openssl
                pkgs.openssl.dev
                pkgs.openssl.bin
              ];
              OPENSSL_NO_VENDOR = 1;
              OPENSSL_DIR="${pkgs.openssl.dev}";
              OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include";
              OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib";
            };
            # working
            librocksdb-sys = attrs: {
              RUST_BACKTRACE="full";
              #LIBCLANG_PATH = "${pkgs.llvmPackages.clang-unwrapped.lib}/lib";
              LIBCLANG_PATH = "${pkgs.clang.cc.lib}/lib";
              ROCKSDB_LIB_DIR = "${pkgs.rocksdb}/lib/";
              C_INCLUDE_PATH = "${pkgs.clang.cc.lib}/lib/clang/17/include/";
              buildInputs = [
                pkgs.pkg-config
              ];
              nativeBuildInputs = [
                pkgs.llvmPackages.clang-unwrapped.lib
                pkgs.pkg-config
                pkgs.breakpointHook
              ];
            };
            # working
            wasm-opt-cxx-sys = attrs: {
              CARGO = cargo;
              LIBCLANG_PATH = "${pkgs.clang.cc.lib}/lib";
              patchPhase = ''
                cp -r ${pkgs.binaryen.src} binaryen
              '';
              CXXFLAGS = "-I${pkgs.binaryen.src}/src -I${pkgs.binaryen.src}/src/tools";
              nativeBuildInputs = [
                pkgs.clang
                pkgs.llvm
                pkgs.pkg-config
                pkgs.breakpointHook
              ];
            };
            litep2p = attrs: {
              buildInputs = [
                pkgs.protobuf
              ];
            };
            sc-network = attrs: {
              CARGO = cargo;
              buildInputs = [
                pkgs.protobuf
              ];
            };
            sc-network-light = attrs: {
              buildInputs = [
                pkgs.protobuf
              ];
            };
            sc-network-sync = attrs: {
              CARGO = cargo;
              buildInputs = [
                pkgs.protobuf
              ];
            };
            # TODO: read the Cargo.toml and just map CARGO to all dependencies listed
            scale-info = attrs: { CARGO = cargo; };
            fork-tree = attrs: { CARGO = cargo; };
            sp-storage = attrs: { CARGO = cargo; };
            sp-tracing = attrs: { CARGO = cargo; };
            sp-version-proc-macro = attrs: { CARGO = cargo; };
            sp-externalities = attrs: { CARGO = cargo; };
            bounded-collections = attrs: { CARGO = cargo; };
            frame-metadata = attrs: { CARGO = cargo; };
            finality-grandpa = attrs: { CARGO = cargo; };
            primitive-types = attrs: { CARGO = cargo; };
            sp-arithmetic = attrs: { CARGO = cargo; };
            sp-metadata-ir = attrs: { CARGO = cargo; };
            sp-weights = attrs: { CARGO = cargo; };
            sp-wasm-interface = attrs: { CARGO = cargo; };
            sp-core = attrs: { CARGO = cargo; };
            sidechain-domain = attrs: { CARGO = cargo; };
            sc-state-db = attrs: { CARGO = cargo; };
            sp-trie = attrs: { CARGO = cargo; };
            selection = attrs: { CARGO = cargo; };
            chain-params = attrs: { CARGO = cargo; };
            sp-io = attrs: { CARGO = cargo; };
            sp-application-crypto = attrs: { CARGO = cargo; };
            sp-runtime = attrs: { CARGO = cargo; };
            epoch-derivation = attrs: { CARGO = cargo; };
            sp-staking = attrs: { CARGO = cargo; };
            sp-version = attrs: { CARGO = cargo; };
            sp-inherents = attrs: { CARGO = cargo; };
            sp-block-rewards = attrs: { CARGO = cargo; };
            sp-api = attrs: { CARGO = cargo; };
            sp-timestamp = attrs: { CARGO = cargo; };
            sp-transaction-storage-proof = attrs: { CARGO = cargo; };
            sp-session-validator = attrs: { CARGO = cargo; };
            sp-session-validator-management = attrs: { CARGO = cargo; };
            frame-system-rpc-runtime-api = attrs: { CARGO = cargo; };
            sp-block-builder = attrs: { CARGO = cargo; };
            sp-genesis-builder = attrs: { CARGO = cargo; };
            sp-consensus-slots = attrs: { CARGO = cargo; };
            sp-consensus-grandpa = attrs: { CARGO = cargo; };
            sp-mixnet = attrs: { CARGO = cargo; };
            sp-offchain = attrs: { CARGO = cargo; };
            sp-session = attrs: { CARGO = cargo; };
            sp-transaction-pool = attrs: { CARGO = cargo; };
            sp-statement-store = attrs: { CARGO = cargo; };
            frame-support = attrs: { CARGO = cargo; };
            sidechain-mc-hash = attrs: { CARGO = cargo; };
            sidechain-slots = attrs: { CARGO = cargo; };
            sp-consensus-aura = attrs: { CARGO = cargo; };
            sp-native-token-management = attrs: { CARGO = cargo; };
            authority-selection-inherents = attrs: { CARGO = cargo; };
            sp-sidechain = attrs: { CARGO = cargo; };
            frame-system = attrs: { CARGO = cargo; };
            pallet-sidechain-rpc = attrs: { CARGO = cargo; };
            sp-session-validator-management-query = attrs: { CARGO = cargo; };
            pallet-session-validator-management-rpc = attrs: { CARGO = cargo; };
            pallet-block-rewards = attrs: { CARGO = cargo; };
            pallet-native-token-management = attrs: { CARGO = cargo; };
            pallet-session-validator-management = attrs: { CARGO = cargo; };
            pallet-sidechain = attrs: { CARGO = cargo; };
            pallet-sudo = attrs: { CARGO = cargo; };
            pallet-transaction-payment = attrs: { CARGO = cargo; };
            pallet-transaction-payment-rpc = attrs: { CARGO = cargo; };
            pallet-partner-chains-session = attrs: { CARGO = cargo; };
            pallet-grandpa = attrs: { CARGO = cargo; };
            pallet-aura = attrs: { CARGO = cargo; };
            pallet-balances = attrs: { CARGO = cargo; };
            pallet-transaction-payment-rpc-runtime-api = attrs: { CARGO = cargo; };
            pallet-authorship = attrs: { CARGO = cargo; };
            pallet-timestamp = attrs: { CARGO = cargo; };
            frame-benchmarking = attrs: { CARGO = cargo; };
            pallet-session = attrs: { CARGO = cargo; };
            sc-client-db = attrs: { CARGO = cargo; };
            sc-network-common = attrs: { CARGO = cargo; };
            sc-mixnet = attrs: { CARGO = cargo; };
            sc-rpc-api = attrs: { CARGO = cargo; };
            sc-consensus-grandpa = attrs: { CARGO = cargo; };
            substrate-frame-rpc-system = attrs: { CARGO = cargo; };
            sc-consensus-grandpa-rpc = attrs: { CARGO = cargo; };
            sc-rpc-spec-v2 = attrs: { CARGO = cargo; };
            sc-cli = attrs: { CARGO = cargo; };
            mock-types = attrs: { CARGO = cargo; };
            cli-commands = attrs: {
              CARGO = cargo;
              WASM_BUILD_WORKSPACE_HINT = "${self}";
              CARGO_MANIFEST_DIR="${self}";
              patchPhase = ''
                echo ${builtins.toJSON attrs}
              '';
            };
            # not working
            sidechain-runtime = attrs: {
              buildInputs = [

              ];
              CARGO_MANIFEST_DIR="${self}";
              CARGO = cargo;
              #WASM_BUILD_WORKSPACE_HINT = "${self}";
              CARGO_NET_OFFLINE = "true";
              patchPhase = ''
                echo ${builtins.toJSON attrs}
                echo $WASM_BUILD_WORKSPACE_HINT
              '';
              nativeBuildInputs = [pkgs.breakpointHook];
              #__noChroot = true;
            };
          };
        };
        rustToolchainToml = pkgs.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml;
        crate2nixPkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            (self: _: {
              rustc = rustToolchainToml;
              cargo = rustToolchainToml;
            })
          ];
        };
        generatedBuild = crate2nixPkgs.callPackage ../Cargo.nix {
          buildRustCrateForPkgs = customBuildRustCrateForPkgs;
        };
      in generatedBuild.workspaceMembers.partner-chains-node.build;
    };
  };
}
