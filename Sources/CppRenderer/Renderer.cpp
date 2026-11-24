#include "Renderer.hpp"
#include "dlbronz_render.h"
#include <cctype>
#include <cstdlib>
#include <cstring>

namespace dlbronz {
std::string renderHTMLText(const std::string& html) {
    std::string out;
    bool inTag = false;
    bool pendingSpace = false;

    for (char ch : html) {
        if (ch == '<') {
            inTag = true;
            pendingSpace = true;
            continue;
        }
        if (ch == '>') {
            inTag = false;
            continue;
        }
        if (inTag) continue;

        if (std::isspace(static_cast<unsigned char>(ch))) {
            pendingSpace = true;
            continue;
        }
        if (pendingSpace && !out.empty()) out += ' ';
        pendingSpace = false;
        out += ch;
    }
    return out;
}
}

extern "C" char *dlbronz_render_html_text(const char *html) {
    if (!html) return nullptr;
    const std::string rendered = dlbronz::renderHTMLText(html);
    char *result = static_cast<char *>(std::malloc(rendered.size() + 1));
    if (!result) return nullptr;
    std::memcpy(result, rendered.c_str(), rendered.size() + 1);
    return result;
}
