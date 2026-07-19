use crate::{
    background::SPLIT_CHAR,
    nostd_types::{EventType, FOOTER, HEADER},
    types::HidEvent,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// All payload structs are `#[serde(default)]` so that a server sending fewer
// fields than we model (different versions, different setups) degrades to
// defaults instead of failing to parse the whole event.

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AuthEventPayload {
    pub api_key: String,
    pub connections: Vec<Connection>,
    pub current_connection_id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Connection {
    pub channel_infos: ChannelInfos,
    pub client_id: i64,
    pub client_infos: Vec<ClientInfo>,
    pub id: i64,
    pub properties: ServerProperties,
    pub status: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ChannelInfos {
    pub root_channels: Vec<Channel>,
    /// Sub-channels keyed by parent channel id (the ids vary per server, so
    /// this must stay a map rather than named fields).
    pub sub_channels: HashMap<String, Vec<Channel>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Channel {
    pub id: String,
    pub order: String,
    pub parent_id: String,
    pub properties: ChannelProperties,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ChannelProperties {
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
#[serde(rename_all = "camelCase", default)]
pub struct ClientInfo {
    pub channel_id: String,
    pub id: i64,
    pub properties: ClientProperties,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ClientProperties {
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
#[serde(rename_all = "camelCase", default)]
pub struct ServerProperties {
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
#[serde(rename_all = "camelCase", default)]
pub struct Status {
    pub code: i64,
    pub message: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MovedEventPayload {
    pub client_id: i64,
    pub connection_id: i64,
    pub hot_reload: bool,
    pub new_channel_id: String,
    pub old_channel_id: String,
    pub properties: ClientProperties,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub visibility: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SpeakingStateChangedEventPayload {
    pub connection_id: i64,
    pub client_id: i64,
    pub client_name: String,
    pub client_unique_id: String,
    pub is_speaking: bool,
    pub whisper_id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ClientUpdatedPayload {
    pub connection_id: i64,
    pub client_id: i64,
    pub client_name: String,
    pub client_unique_id: String,
    pub client_input_muted: bool,
    pub client_output_muted: bool,
    pub client_input_hardware: bool,
    pub client_output_hardware: bool,
    pub client_away: bool,
    pub client_away_message: String,
    pub client_talk_power: i64,
    pub client_talk_request: i64,
    pub client_talk_request_message: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ClientLeftPayload {
    pub connection_id: i64,
    pub client_id: i64,
    pub client_name: String,
    pub client_unique_id: String,
    pub reason_id: i64,
    pub reason_message: String,
    pub invoke_client_id: i64,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ClientEnterPayload {
    pub connection_id: i64,
    pub client_id: i64,
    pub client_name: String,
    pub client_unique_id: String,
    pub client_type: i64,
    pub client_away: bool,
    pub client_away_message: String,
    pub client_talk_power: i64,
    pub client_talk_request: i64,
    pub client_talk_request_message: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MessageRecvPayload {
    pub connection_id: i64,
    pub invoker: Invoker,
    pub message: String,
    pub target_id: i64,
    pub target_mode: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Invoker {
    pub id: i64,
    pub nickname: String,
    pub uid: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TalkStatusChangedPayload {
    pub connection_id: i64,
    pub client_id: i64,
    pub status: i64,
    pub is_whisper: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ClientPropertiesChangedPayload {
    pub client_id: i64,
    pub connection_id: i64,
    pub properties: ClientProperties,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum WsEvent {
    #[serde(rename = "auth")]
    Auth {
        #[serde(default)]
        payload: AuthEventPayload,
        #[serde(default)]
        status: Status,
    },
    #[serde(rename = "clientMoved")]
    NotifyClientMoved {
        #[serde(default)]
        payload: MovedEventPayload,
    },
    #[serde(rename = "notifyClientSpeakingStateChanged")]
    NotifyClientSpeakingStateChanged {
        #[serde(default)]
        payload: SpeakingStateChangedEventPayload,
    },
    #[serde(rename = "clientPropertiesUpdated")]
    ClientPropertiesChanged {
        #[serde(default)]
        payload: ClientPropertiesChangedPayload,
    },

    #[serde(rename = "talkStatusChanged")]
    TalkStatusChanged {
        #[serde(default)]
        payload: TalkStatusChangedPayload,
    },

    #[serde(rename = "notifyClientUpdated")]
    NotifyClientUpdated {
        #[serde(default)]
        payload: ClientUpdatedPayload,
    },
    #[serde(rename = "notifyClientLeftView")]
    NotifyClientLeftView {
        #[serde(default)]
        payload: ClientLeftPayload,
    },
    #[serde(rename = "notifyClientEnterView")]
    NotifyClientEnterView {
        #[serde(default)]
        payload: ClientEnterPayload,
    },
    #[serde(rename = "textMessage")]
    NotifyTextMessageReceived {
        #[serde(default)]
        payload: MessageRecvPayload,
    },
    #[serde(other)]
    Unknown,
}

pub struct Ts6HidEvent {
    pub nickname: String,
    pub message: Option<String>,
    pub talking: bool,
    pub show: bool,
    pub is_self: bool,
}

impl HidEvent for Ts6HidEvent {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.nickname.as_bytes());
        bytes.push(SPLIT_CHAR);
        if let Some(msg) = &self.message {
            bytes.extend_from_slice(msg.as_bytes());
        }
        bytes.push(SPLIT_CHAR);
        bytes.push(if self.talking { 1 } else { 0 });
        bytes.push(SPLIT_CHAR);
        bytes.push(if self.show { 1 } else { 0 });
        bytes.push(SPLIT_CHAR);
        bytes.push(if self.is_self { 1 } else { 0 });

        bytes
    }

    fn chunks(&self) -> Vec<Vec<u8>> {
        let buffer = self.to_bytes();
        let chunk_size = 32;
        let mut offset = 0;
        let mut chunks = Vec::new();
        let mut header_chunk = Vec::new();
        header_chunk.extend_from_slice(&HEADER);
        header_chunk.extend_from_slice(&[EventType::TS6 as u8]);

        chunks.push(header_chunk);
        while offset < buffer.len() {
            let mut chunk = Vec::new();

            let end = std::cmp::min(offset + chunk_size, buffer.len());
            chunk.extend_from_slice(&buffer[offset..end]);
            if chunk.len() < 32 {
                chunk.resize(32, 0);
            }
            chunks.push(chunk);
            offset += chunk_size;
        }
        let mut footer_chunk = Vec::new();
        footer_chunk.extend_from_slice(&FOOTER);
        chunks.push(footer_chunk);

        chunks
    }

    fn event_type(&self) -> crate::nostd_types::EventType {
        crate::nostd_types::EventType::TS6
    }
}
