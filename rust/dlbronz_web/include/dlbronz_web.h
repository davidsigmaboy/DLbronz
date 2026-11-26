#ifndef DLBRONZ_WEB_H
#define DLBRONZ_WEB_H

#ifdef __cplusplus
extern "C" {
#endif

/* Width in px of a text run. Used by layout for line breaking. */
typedef float (*dlbronz_measure_fn)(const char *text, float size, int bold, int italic, int mono);

/* All returned strings are heap allocated; release with dlbronz_web_free. NULL on failure. */
char *dlbronz_web_parse_json(const char *html);
char *dlbronz_web_parse_fragment_json(const char *html);
char *dlbronz_web_render(const char *html, const char *css, float width, dlbronz_measure_fn measure);
void dlbronz_web_free(char *p);

#ifdef __cplusplus
}
#endif

#endif
