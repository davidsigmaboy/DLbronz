#include "dlbronz_c.h"
#include <string.h>

int dlbronz_is_http_url(const char *url) {
    if (url == NULL) return 0;
    return strncmp(url, "https://", 8) == 0 ||
           strncmp(url, "http://", 7) == 0;
}

const char *dlbronz_engine_version(void) {
    return "DLbronz 0.1-multilang";
}
