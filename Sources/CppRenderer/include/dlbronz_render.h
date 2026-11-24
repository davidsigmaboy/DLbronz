#ifndef DLBRONZ_RENDER_H
#define DLBRONZ_RENDER_H
#ifdef __cplusplus
extern "C" {
#endif
/* Returns a malloc'ed, NUL-terminated string with the visible text of the
   given HTML. The caller must free() it. Returns NULL on failure. */
char *dlbronz_render_html_text(const char *html);
#ifdef __cplusplus
}
#endif
#endif
