// Firedancer C Library Wrapper for Bindgen
// This header exposes the core Firedancer functions we want to bind

// Ed25519 signature verification
int fd_ed25519_verify(
    const unsigned char *sig,      // 64-byte signature
    const unsigned char *msg,      // message bytes
    unsigned long msg_sz,          // message size
    const unsigned char *public_key, // 32-byte public key
    void *sha                      // SHA context (can be null)
);

// SHA256 hashing
int fd_sha256_hash(
    const unsigned char *msg,      // input message
    unsigned long msg_sz,          // message length
    unsigned char *hash_out        // 32-byte output buffer
);

// Blake3 hashing
int fd_blake3_hash(
    const unsigned char *msg,      // input message
    unsigned long msg_sz,          // message length
    unsigned char *hash_out        // 32-byte output buffer
);

// sBPF Virtual Machine
void* fd_sbpf_vm_new(void);
void fd_sbpf_vm_delete(void *vm);

// sBPF program management
void* fd_sbpf_program_new(
    const unsigned char *bytecode, // ELF bytecode
    unsigned long bytecode_sz,     // bytecode size
    unsigned long *entry_pc        // entry point output
);

int fd_sbpf_vm_exec(
    void *vm,                      // VM handle
    void *program,                 // program handle
    unsigned char *input,          // input data
    unsigned long input_sz,        // input size
    unsigned char *output,         // output buffer
    unsigned long *output_sz       // output size (in/out)
);

void fd_sbpf_program_delete(void *program);

// Account management
void* fd_acc_mgr_new(void);
void fd_acc_mgr_delete(void *mgr);

int fd_acc_mgr_view(
    void *mgr,
    const unsigned char *address,  // 32-byte account address
    void *record                   // account record output
);

int fd_acc_mgr_modify(
    void *mgr,
    const unsigned char *address,  // 32-byte account address
    const void *record             // account record input
); 