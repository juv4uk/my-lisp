#ifndef MY_LISP_EMBED_H
#define MY_LISP_EMBED_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define MY_LISP_EMBED_ABI_VERSION 1u

typedef struct MyLispEmbedSession MyLispEmbedSession;

uint32_t my_lisp_embed_abi_version(void);
MyLispEmbedSession *my_lisp_embed_session_new(void);
char *my_lisp_embed_eval(MyLispEmbedSession *session, const char *utf8_source);
void my_lisp_embed_free_string(char *value);
void my_lisp_embed_session_free(MyLispEmbedSession *session);

#ifdef __cplusplus
}
#endif

#endif
