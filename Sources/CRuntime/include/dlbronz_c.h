#ifndef DLBRONZ_C_H
#define DLBRONZ_C_H
#ifdef __cplusplus
extern "C" {
#endif
int dlbronz_is_http_url(const char *url);
const char *dlbronz_engine_version(void);
#ifdef __cplusplus
}
#endif
#endif
