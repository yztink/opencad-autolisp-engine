import ctypes
import os

dll_path = os.path.abspath("target/debug/opencad_lisp.dll")
assert os.path.exists(dll_path), f"DLL not found at {dll_path}"

dll = ctypes.CDLL(dll_path)
dll.lisp_engine_create.restype = ctypes.c_void_p
dll.lisp_engine_destroy.argtypes = [ctypes.c_void_p]

# Callback signature: int (*callback)(const char* cmd)
CMD_CALLBACK = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.c_char_p)
dll.lisp_engine_set_command_callback.argtypes = [ctypes.c_void_p, CMD_CALLBACK]

engine = dll.lisp_engine_create()
assert engine is not None, "Failed to create Lisp engine"

executed_commands = []

@CMD_CALLBACK
def on_command(cmd_bytes):
    cmd_str = cmd_bytes.decode("utf-8")
    print(f"  [Host CAD Received]: '{cmd_str}'")
    executed_commands.append(cmd_str)
    return 0

dll.lisp_engine_set_command_callback(ctypes.c_void_p(engine), on_command)

buf = ctypes.create_string_buffer(1024)

# 1. Test math
res = dll.lisp_eval_string(ctypes.c_void_p(engine), b"(+ 10 (* 20 3))", buf, 1024)
assert res == 0, "Math eval failed"
print("1. Math result:", buf.value.decode("utf-8"))

# 2. Test multiline setq
code = b"""
(setq a 100
      b 250
      c (+ a b))
"""
res = dll.lisp_eval_string(ctypes.c_void_p(engine), code, buf, 1024)
assert res == 0, "Setq eval failed"
print("2. Setq result:", buf.value.decode("utf-8"))

# 3. Test (command ...) calling into host callback
cmd_lisp = b'(command "_.CIRCLE" \'(10.0 20.0 0.0) 45.5)'
res = dll.lisp_eval_string(ctypes.c_void_p(engine), cmd_lisp, buf, 1024)
assert res == 0, "Command eval failed"
assert len(executed_commands) == 1, "Expected 1 executed CAD command"
print("3. CAD command successfully executed via callback!")

# 4. Test Visual LISP
res = dll.lisp_eval_string(ctypes.c_void_p(engine), b'(vl-string-subst "CAD" "LISP" "OpenLISP")', buf, 1024)
assert res == 0, "VL eval failed"
print("4. Visual LISP result:", buf.value.decode("utf-8"))

dll.lisp_engine_destroy(ctypes.c_void_p(engine))
print("\nSuccess: opencad_lisp.dll passed all FFI and CAD callback tests!")
