const error = new Error("/lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.39` not found");
error.code = "ERR_DLOPEN_FAILED";
throw error;
