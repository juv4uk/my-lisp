#ifndef MY_LISP_EMBED_H
#define MY_LISP_EMBED_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define MY_LISP_EMBED_ABI_VERSION 3u

typedef struct MyLispEmbedSession MyLispEmbedSession;
typedef int32_t (*MyLispEmbedNullaryFn)(void *context, uint32_t *out_result);

enum {
    MY_LISP_EMBED_NIL = 0u,
    MY_LISP_EMBED_TRUE = 1u,
};

uint32_t my_lisp_embed_abi_version(void);
MyLispEmbedSession *my_lisp_embed_session_new(void);
int32_t my_lisp_embed_bind_host_handle(
    MyLispEmbedSession *session,
    const char *utf8_surface,
    const char *utf8_kind,
    uint64_t token);
int32_t my_lisp_embed_register_nullary(
    MyLispEmbedSession *session,
    const char *utf8_surface,
    MyLispEmbedNullaryFn callback,
    void *context);
char *my_lisp_embed_eval(MyLispEmbedSession *session, const char *utf8_source);
void my_lisp_embed_free_string(char *value);
void my_lisp_embed_session_free(MyLispEmbedSession *session);

#ifdef __cplusplus
}
#endif

#endif
