use crate::dm_channel::DmChannel;
use crate::nip46::{Approval, ParsedCommand};
use crate::people::PersonList;
use crate::relay::Relay;
use nostr_types::{
    Event, EventAddr, Id, IdHex, Metadata, MilliSatoshi, Profile, PublicKey, RelayUrl, Tag,
    UncheckedUrl, Unixtime,
};
use std::fmt;

/// This is a message sent to the Overlord. Tasks which take any amount of time,
/// especially involving relays, are handled by the Overlord in this way. There is
/// no return value, you'll have to check various GLOBALS state later on if you
/// depend on the result. Such an architecture works best with an immediate-mode
/// renderer.
#[derive(Debug, Clone)]
pub enum ToOverlordMessage {
    /// Calls [add_pubkey_relay](crate::Overlord::add_pubkey_relay)
    AddPubkeyRelay(PublicKey, RelayUrl),

    /// Calls [add_relay](crate::Overlord::add_relay)
    AddRelay(RelayUrl),

    /// Calls [advertise_relay_list](crate::Overlord::advertise_relay_list)
    AdvertiseRelayList,

    /// internal
    AdvertiseRelayListNextChunk(Box<Event>, Vec<RelayUrl>),

    /// Calls [auth_approved](crate::Overlord::auth_approved)
    AuthApproved(RelayUrl),

    /// Calls [auth_approved](crate::Overlord::auth_declined)
    AuthDeclined(RelayUrl),

    /// Calls [change_passphrase](crate::Overlord::change_passphrase)
    ChangePassphrase { old: String, new: String },

    /// Calls [clear_person_list](crate::Overlord::clear_person_list)
    ClearPersonList(PersonList),

    /// Calls [auth_approved](crate::Overlord::connect_approved)
    ConnectApproved(RelayUrl),

    /// Calls [auth_approved](crate::Overlord::connect_declined)
    ConnectDeclined(RelayUrl),

    /// Calls [delegation_reset](crate::Overlord::delegation_reset)
    DelegationReset,

    /// Calls [delete_person_list](crate::Overlord::delete_person_list)
    DeletePersonList(PersonList),

    /// Calls [delete_post](crate::Overlord::delete_post)
    DeletePost(Id),

    /// Calls [delete_priv](crate::Overlord::delete_priv)
    DeletePriv,

    /// Calls [delete_pub](crate::Overlord::delete_pub)
    DeletePub,

    /// Calls [drop_relay](crate::Overlord::drop_relay)
    DropRelay(RelayUrl),

    /// Calls [fetch_event](crate::Overlord::fetch_event)
    FetchEvent(Id, Vec<RelayUrl>),

    /// Calls [fetch_event_addr](crate::Overlord::fetch_event_addr)
    FetchEventAddr(EventAddr),

    /// Calls [follow_pubkey](crate::Overlord::follow_pubkey)
    FollowPubkey(PublicKey, PersonList, bool),

    /// Calls [follow_nip05](crate::Overlord::follow_nip05)
    FollowNip05(String, PersonList, bool),

    /// Calls [follow_nprofile](crate::Overlord::follow_nprofile)
    FollowNprofile(Profile, PersonList, bool),

    /// Calls [generate_private_key](crate::Overlord::generate_private_key)
    GeneratePrivateKey(String),

    /// Calls [hide_or_show_relay](crate::Overlord::hide_or_show_relay)
    HideOrShowRelay(RelayUrl, bool),

    /// Calls [import_priv](crate::Overlord::import_priv)
    ImportPriv {
        // nsec, hex, or ncryptsec
        privkey: String,
        password: String,
    },

    /// Calls [import_pub](crate::Overlord::import_pub)
    ImportPub(String),

    /// Calls [like](crate::Overlord::like)
    Like(Id, PublicKey),

    /// Calls [load_more_current_feed](crate::Overlord::load_more_current_feed)
    LoadMoreCurrentFeed,

    /// internal (minions use this channel too)
    MinionJobComplete(RelayUrl, u64),

    /// internal (minions use this channel too)
    MinionJobUpdated(RelayUrl, u64, u64),

    /// Calls [nip46_server_op_approval_response](crate::Overlord::nip46_server_op_approval_response)
    Nip46ServerOpApprovalResponse(PublicKey, ParsedCommand, Approval),

    /// Calls [post](crate::Overlord::post)
    Post {
        content: String,
        tags: Vec<Tag>,
        in_reply_to: Option<Id>,
        dm_channel: Option<DmChannel>,
    },

    /// Calls [post_nip46_event](crate::Overlord::post_nip46_event)
    PostNip46Event(Event, Vec<RelayUrl>),

    /// Calls [prune_cache](crate::Overlord::prune_cache)
    PruneCache,

    /// Calls [prune_database](crate::Overlord::prune_database)
    PruneDatabase,

    /// Calls [push_person_list](crate::Overlord::push_person_list)
    PushPersonList(PersonList),

    /// Calls [push_metadata](crate::Overlord::push_metadata)
    PushMetadata(Metadata),

    /// Calls [rank_relay](crate::Overlord::rank_relay)
    RankRelay(RelayUrl, u8),

    /// internal (the overlord sends messages to itself sometimes!)
    ReengageMinion(RelayUrl, Vec<RelayJob>),

    /// Calls [refresh_scores_and_pick_relays](crate::Overlord::refresh_scores_and_pick_relays)
    RefreshScoresAndPickRelays,

    /// Calls [reresh_subscribed_metadata](crate::Overlord::refresh_subscribed_metadata)
    RefreshSubscribedMetadata,

    /// Calls [repost](crate::Overlord::repost)
    Repost(Id),

    /// Calls [search](crate::Overlord::search)
    Search(String),

    /// Calls [set_active_person](crate::Overlord::set_active_person)
    SetActivePerson(PublicKey),

    /// internal
    SetDmChannel(DmChannel),

    /// internal
    SetPersonFeed(PublicKey),

    /// internal
    SetThreadFeed {
        id: Id,
        referenced_by: Id,
        author: Option<PublicKey>,
    },

    /// Calls [start_long_lived_subscriptions](crate::Overlord::start_long_lived_subscriptions)
    StartLongLivedSubscriptions,

    /// Calls [subscribe_config](crate::Overlord::subscribe_config)
    SubscribeConfig(Option<Vec<RelayUrl>>),

    /// Calls [subscribe_discover](crate::Overlord::subscribe_discover)
    SubscribeDiscover(Vec<PublicKey>, Option<Vec<RelayUrl>>),

    /// Calls [subscribe_mentions](crate::Overlord::subscribe_mentions)
    SubscribeMentions(Option<Vec<RelayUrl>>),

    /// Calls [subscribe_nip46](crate::Overlord::subscribe_nip46)
    SubscribeNip46(Vec<RelayUrl>),

    /// Calls [shutdown](crate::Overlord::shutdown)
    Shutdown,

    /// Calls [unlock_key](crate::Overlord::unlock_key)
    UnlockKey(String),

    /// Calls [update_metadata](crate::Overlord::update_metadata)
    UpdateMetadata(PublicKey),

    /// Calls [update_metadata_in_bulk](crate::Overlord::update_metadata_in_bulk)
    UpdateMetadataInBulk(Vec<PublicKey>),

    /// Calls [update_person_list](crate::Overlord::update_person_list)
    UpdatePersonList {
        person_list: PersonList,
        merge: bool,
    },

    /// Calls [update_relay](crate::Overlord::update_relay)
    UpdateRelay(Relay, Relay),

    /// Calls [visible_notes_changed](crate::Overlord::visible_notes_changed)
    VisibleNotesChanged(Vec<Id>),

    /// Calls [zap_start](crate::Overlord::zap_start)
    ZapStart(Id, PublicKey, UncheckedUrl),

    /// Calls [zap](crate::Overlord::zap)
    Zap(Id, PublicKey, MilliSatoshi, String),
}

/// Internal to gossip-lib.
/// This is a message sent to the minions
#[derive(Debug, Clone)]
pub(crate) struct ToMinionMessage {
    /// The minion we are addressing, based on the URL they are listening to
    /// as a String.  "all" means all minions.
    pub target: String,

    pub payload: ToMinionPayload,
}

#[derive(Debug, Clone)]
pub(crate) struct ToMinionPayload {
    /// A job id, so the minion and overlord can talk about the job.
    pub job_id: u64,

    pub detail: ToMinionPayloadDetail,
}

#[derive(Debug, Clone)]
pub(crate) enum ToMinionPayloadDetail {
    AdvertiseRelayList(Box<Event>),
    AuthApproved,
    AuthDeclined,
    FetchEvent(Id),
    FetchEventAddr(EventAddr),
    PostEvent(Box<Event>),
    Shutdown,
    SubscribeAugments(Vec<IdHex>),
    SubscribeOutbox,
    SubscribeDiscover(Vec<PublicKey>),
    SubscribeGeneralFeed(Vec<PublicKey>),
    SubscribeMentions,
    SubscribePersonFeed(PublicKey),
    SubscribeThreadFeed(IdHex, Vec<IdHex>),
    SubscribeDmChannel(DmChannel),
    SubscribeNip46,
    TempSubscribeGeneralFeedChunk {
        pubkeys: Vec<PublicKey>,
        start: Unixtime,
    },
    TempSubscribePersonFeedChunk {
        pubkey: PublicKey,
        start: Unixtime,
    },
    TempSubscribeInboxFeedChunk(Unixtime),
    TempSubscribeMetadata(Vec<PublicKey>),
    UnsubscribePersonFeed,
    UnsubscribeThreadFeed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayConnectionReason {
    Advertising,
    Config,
    Discovery,
    FetchAugments,
    FetchDirectMessages,
    FetchContacts,
    FetchEvent,
    FetchMentions,
    FetchMetadata,
    Follow,
    NostrConnect,
    PostEvent,
    PostContacts,
    PostLike,
    PostMetadata,
    PostMuteList,
    PostNostrConnect,
    ReadThread,
    SubscribePerson,
}

impl fmt::Display for RelayConnectionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)?;
        Ok(())
    }
}

impl RelayConnectionReason {
    pub fn description(&self) -> &'static str {
        use RelayConnectionReason::*;
        match *self {
            Discovery => "Searching for other people's Relay Lists",
            Config => "Reading our client configuration",
            FetchMentions => "Searching for mentions of us",
            Follow => "Following the posts of people in our Contact List",
            FetchAugments => "Fetching events that augment other events (likes, zaps, deletions)",
            FetchDirectMessages => "Fetching direct messages",
            FetchEvent => "Fetching a particular event",
            FetchMetadata => "Fetching metadata for a person",
            NostrConnect => "Nostr connect",
            PostEvent => "Posting an event",
            Advertising => "Advertising our relay list",
            PostLike => "Posting a reaction to an event",
            FetchContacts => "Fetching our contact list",
            PostContacts => "Posting our contact list",
            PostMuteList => "Posting our mute list",
            PostMetadata => "Posting our metadata",
            PostNostrConnect => "Posting nostrconnect",
            ReadThread => "Reading ancestors to build a thread",
            SubscribePerson => "Subscribe to the events of a person",
        }
    }

    pub fn persistent(&self) -> bool {
        use RelayConnectionReason::*;
        match *self {
            Discovery => false,
            Config => false,
            FetchMentions => true,
            Follow => true,
            FetchAugments => false,
            FetchDirectMessages => true,
            FetchEvent => false,
            FetchMetadata => false,
            NostrConnect => true,
            PostEvent => false,
            Advertising => false,
            PostLike => false,
            FetchContacts => false,
            PostContacts => false,
            PostMuteList => false,
            PostMetadata => false,
            PostNostrConnect => false,
            ReadThread => true,
            SubscribePerson => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RelayJob {
    // Short reason for human viewing
    pub reason: RelayConnectionReason,

    // Payload sent when it was started
    pub(crate) payload: ToMinionPayload,
    // NOTE, there is other per-relay data stored elsewhere in
    //   overlord.minions_task_url
    //   GLOBALS.relay_picker
}
