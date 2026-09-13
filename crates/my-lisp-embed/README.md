# my-lisp-embed

`my-lisp-embed` is the narrow C ABI boundary for hosts that need a real,
stateful `my-lisp` session.  It delegates parsing, bootstrap, evaluation,
printing, errors, definitions, macros, and closures to the canonical
`my-lisp` crate; it contains no evaluator or alternate language semantics.

The ABI is intentionally small:

```c
MyLispEmbedSession *my_lisp_embed_session_new(void);
char *my_lisp_embed_eval(MyLispEmbedSession *, const char *utf8_source);
void my_lisp_embed_free_string(char *);
void my_lisp_embed_session_free(MyLispEmbedSession *);
```

`my_lisp_embed_eval` evaluates a complete UTF-8 source fragment against the
same session on every call.  The caller owns a non-null returned string and
releases it with `my_lisp_embed_free_string`.  A null session produces a
null pointer.  Parse and language failures are returned as human-readable
`error:` lines, rather than crossing the ABI as panics.

This crate deliberately does **not** define Cyberpunk capabilities.  A host
capability bridge belongs in a later, explicit contract so the RED4ext adapter
remains mechanism while my-lisp retains semantic authority.
