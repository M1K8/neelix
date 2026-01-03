use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthEventPayload {
    pub api_key: String,
    pub connections: Vec<Connection>,
    pub current_connection_id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub channel_infos: ChannelInfos,
    pub client_id: i64,
    pub client_infos: Vec<ClientInfo>,
    pub id: i64,
    pub properties: Properties6,
    pub status: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelInfos {
    pub root_channels: Vec<RootChannel>,
    pub sub_channels: SubChannels,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootChannel {
    pub id: String,
    pub order: String,
    pub parent_id: String,
    pub properties: Properties,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties {
    pub banner_gfx_url: String,
    pub banner_mode: i64,
    pub codec: i64,
    pub codec_is_unencrypted: bool,
    pub codec_latency_factor: i64,
    pub codec_quality: i64,
    pub delete_delay: i64,
    pub description: String,
    pub flag_are_subscribed: bool,
    pub flag_default: bool,
    pub flag_maxclients_unlimited: bool,
    pub flag_maxfamilyclients_inherited: bool,
    pub flag_maxfamilyclients_unlimited: bool,
    pub flag_password: bool,
    pub flag_permanent: bool,
    pub flag_semi_permanent: bool,
    pub forced_silence: bool,
    pub icon_id: i64,
    pub maxclients: i64,
    pub maxfamilyclients: i64,
    pub name: String,
    pub name_phonetic: String,
    pub needed_talk_power: i64,
    pub order: String,
    pub permission_hints: i64,
    pub storage_quota: i64,
    pub topic: String,
    pub unique_identifier: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubChannels {
    #[serde(rename = "10")]
    pub n10: Vec<n10>,
    #[serde(rename = "18")]
    pub n18: Vec<n18>,
    #[serde(rename = "76")]
    pub n76: Vec<n76>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct n10 {
    pub id: String,
    pub order: String,
    pub parent_id: String,
    pub properties: Properties2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties2 {
    pub banner_gfx_url: String,
    pub banner_mode: i64,
    pub codec: i64,
    pub codec_is_unencrypted: bool,
    pub codec_latency_factor: i64,
    pub codec_quality: i64,
    pub delete_delay: i64,
    pub description: String,
    pub flag_are_subscribed: bool,
    pub flag_default: bool,
    pub flag_maxclients_unlimited: bool,
    pub flag_maxfamilyclients_inherited: bool,
    pub flag_maxfamilyclients_unlimited: bool,
    pub flag_password: bool,
    pub flag_permanent: bool,
    pub flag_semi_permanent: bool,
    pub forced_silence: bool,
    pub icon_id: i64,
    pub maxclients: i64,
    pub maxfamilyclients: i64,
    pub name: String,
    pub name_phonetic: String,
    pub needed_talk_power: i64,
    pub order: String,
    pub permission_hints: i64,
    pub storage_quota: i64,
    pub topic: String,
    pub unique_identifier: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct n18 {
    pub id: String,
    pub order: String,
    pub parent_id: String,
    pub properties: Properties3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties3 {
    pub banner_gfx_url: String,
    pub banner_mode: i64,
    pub codec: i64,
    pub codec_is_unencrypted: bool,
    pub codec_latency_factor: i64,
    pub codec_quality: i64,
    pub delete_delay: i64,
    pub description: String,
    pub flag_are_subscribed: bool,
    pub flag_default: bool,
    pub flag_maxclients_unlimited: bool,
    pub flag_maxfamilyclients_inherited: bool,
    pub flag_maxfamilyclients_unlimited: bool,
    pub flag_password: bool,
    pub flag_permanent: bool,
    pub flag_semi_permanent: bool,
    pub forced_silence: bool,
    pub icon_id: i64,
    pub maxclients: i64,
    pub maxfamilyclients: i64,
    pub name: String,
    pub name_phonetic: String,
    pub needed_talk_power: i64,
    pub order: String,
    pub permission_hints: i64,
    pub storage_quota: i64,
    pub topic: String,
    pub unique_identifier: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct n76 {
    pub id: String,
    pub order: String,
    pub parent_id: String,
    pub properties: Properties4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties4 {
    pub banner_gfx_url: String,
    pub banner_mode: i64,
    pub codec: i64,
    pub codec_is_unencrypted: bool,
    pub codec_latency_factor: i64,
    pub codec_quality: i64,
    pub delete_delay: i64,
    pub description: String,
    pub flag_are_subscribed: bool,
    pub flag_default: bool,
    pub flag_maxclients_unlimited: bool,
    pub flag_maxfamilyclients_inherited: bool,
    pub flag_maxfamilyclients_unlimited: bool,
    pub flag_password: bool,
    pub flag_permanent: bool,
    pub flag_semi_permanent: bool,
    pub forced_silence: bool,
    pub icon_id: i64,
    pub maxclients: i64,
    pub maxfamilyclients: i64,
    pub name: String,
    pub name_phonetic: String,
    pub needed_talk_power: i64,
    pub order: String,
    pub permission_hints: i64,
    pub storage_quota: i64,
    pub topic: String,
    pub unique_identifier: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    pub channel_id: String,
    pub id: i64,
    pub properties: Properties5,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties5 {
    pub away: bool,
    pub away_message: String,
    pub badges: String,
    pub channel_group_id: String,
    pub channel_group_inherited_channel_id: String,
    pub country: String,
    pub created: i64,
    pub database_id: String,
    pub default_channel: String,
    pub default_channel_password: String,
    pub default_token: String,
    pub description: String,
    pub flag_avatar: String,
    pub flag_talking: bool,
    pub icon_id: i64,
    pub idle_time: i64,
    pub input_deactivated: bool,
    pub input_hardware: bool,
    pub input_muted: bool,
    pub integrations: String,
    pub is_channel_commander: bool,
    pub is_muted: bool,
    pub is_priority_speaker: bool,
    pub is_recording: bool,
    pub is_streaming: bool,
    pub is_talker: bool,
    pub last_connected: i64,
    pub meta_data: String,
    pub month_bytes_downloaded: i64,
    pub month_bytes_uploaded: i64,
    pub myteamspeak_avatar: String,
    pub myteamspeak_id: String,
    pub needed_server_query_view_power: i64,
    pub nickname: String,
    pub nickname_phonetic: String,
    pub output_hardware: bool,
    pub output_muted: bool,
    pub output_only_muted: bool,
    pub permission_hints: i64,
    pub platform: String,
    pub server_groups: String,
    pub server_password: String,
    pub signed_badges: String,
    pub talk_power: i64,
    pub talk_request: i64,
    pub talk_request_msg: String,
    pub total_bytes_downloaded: i64,
    pub total_bytes_uploaded: i64,
    pub total_connections: i64,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub unique_identifier: String,
    pub unread_messages: i64,
    pub user_tag: String,
    pub version: String,
    pub volume_modificator: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties6 {
    pub address: String,
    pub anti_flood_points_needed_command_block: i64,
    pub anti_flood_points_needed_ip_block: i64,
    pub anti_flood_points_needed_plugin_block: i64,
    pub anti_flood_points_tick_reduce: i64,
    pub ask_for_privilege_key: bool,
    pub autostart: bool,
    pub canonical_name: String,
    pub capability_extensions: String,
    pub channel_temp_delete_delay_default: i64,
    pub channels_online: i64,
    pub client_connections: i64,
    pub clients_online: i64,
    pub codec_encryption_mode: i64,
    pub complain_autoban_count: i64,
    pub complain_autoban_time: i64,
    pub complain_remove_time: i64,
    pub created: i64,
    pub default_channel_admin_group: String,
    pub default_channel_group: String,
    pub default_server_group: String,
    pub download_quota: i64,
    pub flag_password: bool,
    pub homebase_storage_quota: i64,
    pub host_banner_gfx_interval: i64,
    pub host_banner_gfx_url: String,
    pub host_banner_mode: i64,
    pub host_banner_url: String,
    pub host_button_gfx_url: String,
    pub host_button_tooltip: String,
    pub host_button_url: String,
    pub host_message: String,
    pub host_message_mode: i64,
    pub icon_id: i64,
    pub id: String,
    pub ip: String,
    pub log_channel: bool,
    pub log_client: bool,
    pub log_filetransfer: bool,
    pub log_permissions: bool,
    pub log_query: bool,
    pub log_server: bool,
    pub machine_id: String,
    pub max_clients: i64,
    pub max_download_total_bandwidth: String,
    pub max_homebases: i64,
    pub max_upload_total_bandwidth: String,
    pub min_android_version: i64,
    pub min_client_version: i64,
    pub min_clients_in_channel_before_forced_silence: i64,
    pub min_ios_version: i64,
    pub min_winphone_version: i64,
    pub month_bytes_downloaded: i64,
    pub month_bytes_uploaded: i64,
    pub mytsid_connect_only: bool,
    pub name: String,
    pub name_phonetic: String,
    pub needed_identity_security_level: i64,
    pub nickname: String,
    pub platform: String,
    pub port: i64,
    pub priority_speaker_dimm_modificator: f64,
    pub query_client_connections: i64,
    pub query_clients_online: i64,
    pub reserved_slots: i64,
    pub storage_quota: i64,
    pub total_bytes_downloaded: String,
    pub total_bytes_uploaded: String,
    pub total_packet_loss_control: f64,
    pub total_packet_loss_keep_alive: f64,
    pub total_packet_loss_speech: f64,
    pub total_packet_loss_total: f64,
    pub unique_identifier: String,
    pub upload_quota: i64,
    pub uptime: i64,
    pub uuid: String,
    pub version: String,
    pub version_sign: String,
    pub web_list_enabled: bool,
    pub welcome_message: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub code: i64,
    pub message: String,
}
