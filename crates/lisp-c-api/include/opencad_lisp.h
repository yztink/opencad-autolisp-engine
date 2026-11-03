#ifndef OPENCAD_LISP_H
#define OPENCAD_LISP_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct LispEngine LispEngine;

/**
 * Function pointer for host CAD command execution:
 * Returns 0 on success, non-zero on failure.
 */
typedef int (*LispCmdCallback)(const char* cmd_str);

/**
 * Creates a new AutoLISP engine instance.
 * Caller is responsible for destroying it with `lisp_engine_destroy`.
 */
LispEngine* lisp_engine_create(void);

/**
 * Destroys an AutoLISP engine instance and releases all associated memory.
 */
void lisp_engine_destroy(LispEngine* engine);

/**
 * Sets a host callback to handle `(command ...)` expressions.
 */
void lisp_engine_set_command_callback(LispEngine* engine, LispCmdCallback callback);

/**
 * Evaluates an AutoLISP code string.
 *
 * @param engine Pointer to active LispEngine
 * @param code Null-terminated UTF-8 string containing AutoLISP code
 * @param out_buf Optional buffer to receive string representation of the result or error
 * @param out_buf_len Size of out_buf in bytes
 * @return 0 on success, -1 on evaluation error, -2 on null parameters
 */
int lisp_eval_string(LispEngine* engine, const char* code, char* out_buf, size_t out_buf_len);

/**
 * Retrieves the last error message from the engine into out_buf.
 */
int lisp_get_last_error(LispEngine* engine, char* out_buf, size_t out_buf_len);

#ifdef __cplusplus
}
#endif

#endif /* OPENCAD_LISP_H */
