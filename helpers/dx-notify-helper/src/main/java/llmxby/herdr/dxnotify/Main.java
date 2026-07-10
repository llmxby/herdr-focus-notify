package llmxby.herdr.dxnotify;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.sankuai.sso.oidc.client.OAuthClient;
import com.sankuai.sso.oidc.client.OAuthClientConfigBuilder;
import com.sankuai.sso.oidc.client.OAuthClientFactory;
import com.sankuai.sso.oidc.enums.Env;
import com.sankuai.sso.oidc.enums.GrantTypeEnum;
import com.sankuai.sso.oidc.request.OAuthTokenRequest;
import com.sankuai.sso.oidc.request.OAuthTokenRequestBuilder;
import com.sankuai.sso.oidc.response.OAuthTokenResponse;
import okhttp3.MediaType;
import okhttp3.OkHttpClient;
import okhttp3.Request;
import okhttp3.RequestBody;
import okhttp3.Response;

import java.time.Duration;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Minimal Daxiang single-chat helper for herdr-focus-notify away fallback.
 *
 * Required env:
 * - DX_CLIENT_ID
 * - DX_CLIENT_SECRET
 * - HERDR_FOCUS_NOTIFY_TITLE
 * - HERDR_FOCUS_NOTIFY_BODY
 * - HERDR_FOCUS_NOTIFY_RECIPIENTS
 *
 * Optional env:
 * - DX_AUDIENCE (default xm-xai)
 * - HERDR_FOCUS_NOTIFY_STATUS
 * - HERDR_FOCUS_NOTIFY_PANE_ID
 * - HERDR_FOCUS_NOTIFY_GROUP
 */
public final class Main {

    private static final MediaType JSON = MediaType.parse("application/json; charset=utf-8");

    private static final String DX_SEND_CHAT_URL =
            "https://xopen.sankuai.com/open-apis/dx-msg/sendChatMsgByRobot";
    private static final String DX_QUERY_IDENTITY_URL =
            "https://xopen.sankuai.com/open-apis/dx/queryEmpIdentityByMisList";

    private final ObjectMapper objectMapper = new ObjectMapper();
    private final OkHttpClient httpClient = new OkHttpClient.Builder()
            .connectTimeout(Duration.ofSeconds(10))
            .readTimeout(Duration.ofSeconds(10))
            .writeTimeout(Duration.ofSeconds(10))
            .build();

    public static void main(String[] args) throws Exception {
        new Main().run();
    }

    private void run() throws Exception {
        String clientId = requireEnv("DX_CLIENT_ID");
        String clientSecret = requireEnv("DX_CLIENT_SECRET");
        String audience = envOrDefault("DX_AUDIENCE", "xm-xai");

        String title = requireEnv("HERDR_FOCUS_NOTIFY_TITLE");
        String body = requireEnv("HERDR_FOCUS_NOTIFY_BODY");
        String status = envOrDefault("HERDR_FOCUS_NOTIFY_STATUS", "unknown");
        String paneId = envOrDefault("HERDR_FOCUS_NOTIFY_PANE_ID", "unknown");
        String recipientsRaw = requireEnv("HERDR_FOCUS_NOTIFY_RECIPIENTS");
        List<String> recipients = parseRecipients(recipientsRaw);
        if (recipients.isEmpty()) {
            throw new IllegalArgumentException("HERDR_FOCUS_NOTIFY_RECIPIENTS is empty");
        }

        String markdown = buildMarkdown(title, body, status, paneId);
        String accessToken = getAccessToken(clientId, clientSecret, audience);
        List<Long> uidList = resolveUids(accessToken, recipients);
        if (uidList.isEmpty()) {
            throw new IllegalStateException("No Daxiang UID resolved from recipients: " + recipients);
        }

        sendMarkdown(accessToken, uidList, markdown);
        System.out.println("dx-notify-helper: sent to " + recipients);
    }

    private String buildMarkdown(String title, String body, String status, String paneId) {
        StringBuilder sb = new StringBuilder();
        sb.append("### Herdr Focus Notify\n\n");
        sb.append("**").append(escapeMarkdown(title)).append("**\n\n");
        sb.append(escapeMarkdown(body)).append("\n\n");
        sb.append("- status: `").append(escapeBackticks(status)).append("`\n");
        sb.append("- pane: `").append(escapeBackticks(paneId)).append("`\n");
        return sb.toString();
    }

    private void sendMarkdown(String accessToken, List<Long> uidList, String markdown) throws Exception {
        Map<String, Object> bodyContent = Collections.<String, Object>singletonMap("text", markdown);
        Map<String, Object> extensionContent = Collections.<String, Object>singletonMap("fileType", "markdown");

        Map<String, Object> sendMsgInfo = new HashMap<String, Object>();
        sendMsgInfo.put("type", "text");
        sendMsgInfo.put("body", objectMapper.writeValueAsString(bodyContent));
        sendMsgInfo.put("extension", objectMapper.writeValueAsString(extensionContent));

        Map<String, Object> body = new HashMap<String, Object>();
        body.put("receiverIds", uidList);
        body.put("sendMsgInfo", sendMsgInfo);

        Request req = new Request.Builder()
                .url(DX_SEND_CHAT_URL)
                .header("Content-Type", "application/json")
                .header("Authorization", accessToken)
                .post(RequestBody.create(JSON, objectMapper.writeValueAsString(body)))
                .build();
        try (Response resp = httpClient.newCall(req).execute()) {
            String respBody = resp.body() != null ? resp.body().string() : "";
            JsonNode root = objectMapper.readTree(respBody);
            int code = root.path("status").path("code").asInt(-1);
            if (code != 0) {
                throw new IllegalStateException("sendChatMsgByRobot failed: status=" + resp.code() + " body=" + respBody);
            }
        }
    }

    private List<Long> resolveUids(String accessToken, List<String> misList) throws Exception {
        Request req = new Request.Builder()
                .url(DX_QUERY_IDENTITY_URL)
                .header("Content-Type", "application/json")
                .header("Authorization", accessToken)
                .post(RequestBody.create(JSON, objectMapper.writeValueAsString(Collections.<String, Object>singletonMap("misList", misList))))
                .build();
        try (Response resp = httpClient.newCall(req).execute()) {
            String respBody = resp.body() != null ? resp.body().string() : "";
            JsonNode root = objectMapper.readTree(respBody);
            int code = root.path("status").path("code").asInt(-1);
            if (code != 0) {
                throw new IllegalStateException("queryEmpIdentityByMisList failed: status=" + resp.code() + " body=" + respBody);
            }

            JsonNode dataMap = root.path("data").path("data");
            if (!dataMap.isObject()) {
                return Collections.emptyList();
            }
            List<Long> uids = new ArrayList<>();
            dataMap.fields().forEachRemaining(entry -> {
                long uid = entry.getValue().path("uid").asLong(0);
                if (uid > 0) {
                    uids.add(uid);
                }
            });
            return uids;
        }
    }

    private String getAccessToken(String clientId, String clientSecret, String audience) {
        OAuthClientFactory.initialize(Env.Prod);
        OAuthClient oAuthClient = OAuthClientFactory.getOrCreateClient(
                OAuthClientConfigBuilder.create()
                        .clientId(clientId)
                        .clientSecret(clientSecret)
                        .build()
        );

        OAuthTokenRequest request = OAuthTokenRequestBuilder.create()
                .grantType(GrantTypeEnum.ClientCredentials)
                .audience(new String[]{audience})
                .build();
        OAuthTokenResponse response = oAuthClient.requestToken(request);
        String token = response.getAccessToken();
        if (token == null || token.isEmpty()) {
            throw new IllegalStateException("Failed to get Daxiang access token");
        }
        return token;
    }

    private static List<String> parseRecipients(String raw) {
        if (raw == null || raw.trim().isEmpty()) {
            return Collections.emptyList();
        }
        List<String> recipients = new ArrayList<>();
        Arrays.stream(raw.split(","))
                .map(String::trim)
                .filter(s -> !s.isEmpty())
                .forEach(recipients::add);
        return recipients;
    }

    private static String requireEnv(String key) {
        String value = System.getenv(key);
        if (value == null || value.trim().isEmpty()) {
            throw new IllegalArgumentException("Missing required env: " + key);
        }
        return value;
    }

    private static String envOrDefault(String key, String defaultValue) {
        String value = System.getenv(key);
        return value == null || value.trim().isEmpty() ? defaultValue : value;
    }

    private static String escapeMarkdown(String value) {
        return value.replace("\\", "\\\\");
    }

    private static String escapeBackticks(String value) {
        return value.replace("`", "\\`");
    }
}
