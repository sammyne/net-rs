initSidebarItems({"fn":[["parse","parse parses rawurl into a URL structure."],["parse_query","parse_query parses the URL-encoded query string and returns a map listing the values specified for each key. parse_query always returns a non-nil map containing all the valid query parameters found; err describes the first decoding error encountered, if any."],["parse_request_uri","parse_request_uri parses rawurl into a URL structure. It assumes that rawurl was received in an HTTP request, so the rawurl is interpreted only as an absolute URI or an absolute path. The string rawurl is assumed not to have a #fragment suffix. (Web browsers strip #fragment before sending the URL to a web server.)"],["path_escape","path_escape escapes the string so it can be safely placed inside a URL path segment, replacing special characters (including /) with %XX sequences as needed."],["path_unescape","path_unescape does the inverse transformation of path_escape, converting each 3-byte encoded substring of the form \"%AB\" into the hex-decoded byte 0xAB. It returns an error if any % is not followed by two hexadecimal digits."],["query_escape","query_escape escapes the string so it can be safely placed inside a URL query."],["query_unescape","query_unescape does the inverse transformation of query_escape, converting each 3-byte encoded substring of the form \"%AB\" into the hex-decoded byte 0xAB. It returns an error if any % is not followed by two hexadecimal digits."],["user","user returns a Userinfo containing the provided name and no password set."],["user_password","user_password returns a Userinfo containing the provided name and password."]],"mod":[["errors","module errors define errors about URL operations"]],"struct":[["URL","A URL represents a parsed URL (technically, a URI reference)."],["UserInfo","The Userinfo type is an immutable encapsulation of username and password details for a URL. An existing Userinfo value is guaranteed to have a username set (potentially empty, as allowed by RFC 2396), and optionally a password."],["Values","Values maps a string key to a list of values. It is typically used for query parameters and form values. Unlike in the http.Header map, the keys in a Values map are case-sensitive."]]});