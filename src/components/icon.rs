//! Application icon names backed by installed rahti-icons components.

use crate::components::rahti_icons::arrow_down::ArrowDown;
use crate::components::rahti_icons::arrow_right::ArrowRight;
use crate::components::rahti_icons::arrow_up::ArrowUp;
use crate::components::rahti_icons::at_sign::AtSign;
use crate::components::rahti_icons::bell::Bell;
use crate::components::rahti_icons::bell_off::BellOff;
use crate::components::rahti_icons::bold::Bold;
use crate::components::rahti_icons::calendar::Calendar;
use crate::components::rahti_icons::calendar_days::CalendarDays;
use crate::components::rahti_icons::check::Check;
use crate::components::rahti_icons::chevron_down::ChevronDown;
use crate::components::rahti_icons::chevron_left::ChevronLeft;
use crate::components::rahti_icons::chevron_right::ChevronRight;
use crate::components::rahti_icons::chevron_up::ChevronUp;
use crate::components::rahti_icons::circle_check::CircleCheck;
use crate::components::rahti_icons::circle_dot::CircleDot;
use crate::components::rahti_icons::clock::Clock;
use crate::components::rahti_icons::cloud::Cloud;
use crate::components::rahti_icons::contact::Contact;
use crate::components::rahti_icons::copy::Copy;
use crate::components::rahti_icons::ellipsis::Ellipsis;
use crate::components::rahti_icons::eye::Eye;
use crate::components::rahti_icons::face_slightly_smiling::FaceSlightlySmiling;
use crate::components::rahti_icons::file::File;
use crate::components::rahti_icons::file_text::FileText;
use crate::components::rahti_icons::folder::Folder;
use crate::components::rahti_icons::folder_plus::FolderPlus;
use crate::components::rahti_icons::gem::Gem;
use crate::components::rahti_icons::grid_2x2::Grid2x2;
use crate::components::rahti_icons::hand::Hand;
use crate::components::rahti_icons::hash::Hash;
use crate::components::rahti_icons::image::Image;
use crate::components::rahti_icons::italic::Italic;
use crate::components::rahti_icons::link::Link;
use crate::components::rahti_icons::list::List;
use crate::components::rahti_icons::list_filter::ListFilter;
use crate::components::rahti_icons::list_ordered::ListOrdered;
use crate::components::rahti_icons::mail::Mail;
use crate::components::rahti_icons::map_pin::MapPin;
use crate::components::rahti_icons::message_circle::MessageCircle;
use crate::components::rahti_icons::mic::Mic;
use crate::components::rahti_icons::mic_off::MicOff;
use crate::components::rahti_icons::minus::Minus;
use crate::components::rahti_icons::panels_left_bottom::PanelsLeftBottom;
use crate::components::rahti_icons::paperclip::Paperclip;
use crate::components::rahti_icons::paw_print::PawPrint;
use crate::components::rahti_icons::pencil::Pencil;
use crate::components::rahti_icons::phone::Phone;
use crate::components::rahti_icons::pizza::Pizza;
use crate::components::rahti_icons::play::Play;
use crate::components::rahti_icons::plus::Plus;
use crate::components::rahti_icons::repeat::Repeat;
use crate::components::rahti_icons::screen_share::ScreenShare;
use crate::components::rahti_icons::search::Search;
use crate::components::rahti_icons::send::Send;
use crate::components::rahti_icons::settings::Settings;
use crate::components::rahti_icons::shield::Shield;
use crate::components::rahti_icons::sparkles::Sparkles;
use crate::components::rahti_icons::square::Square;
use crate::components::rahti_icons::square_pen::SquarePen;
use crate::components::rahti_icons::thumbs_up::ThumbsUp;
use crate::components::rahti_icons::trash_2::Trash2;
use crate::components::rahti_icons::type_icon::Type;
use crate::components::rahti_icons::upload::Upload;
use crate::components::rahti_icons::user_plus::UserPlus;
use crate::components::rahti_icons::users::Users;
use crate::components::rahti_icons::video::Video;
use crate::components::rahti_icons::video_off::VideoOff;
use crate::components::rahti_icons::volleyball::Volleyball;
use crate::components::rahti_icons::volume_2::Volume2;
use crate::components::rahti_icons::x::X;
use crate::rahti::{Html, component, html};

#[component]
pub fn Icon(name: &str, class: &str) -> Html {
    let leave_class = format!("{class} rotate-[135deg]");
    match name {
        "accessibility" => html! {
            <svg
                class=@{class}
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.6"
                stroke-linecap="round"
                stroke-linejoin="round"
                aria-hidden="true"
            >
                <circle cx="12" cy="4" r="2" />
                <path d="m4 8 8 2 8-2M12 10v5m0 0-5 7m5-7 5 7" />
            </svg>
        },
        "chat" => html! {
            <MessageCircle class=@{class} stroke_width="1.6" />
        },
        "video" => html! {
            <Video class=@{class} stroke_width="1.6" />
        },
        "contacts" => html! {
            <Contact class=@{class} stroke_width="1.6" />
        },
        "communities" => html! {
            <Users class=@{class} stroke_width="1.6" />
        },
        "people" => html! {
            <Users class=@{class} stroke_width="1.6" />
        },
        "calendar" => html! {
            <Calendar class=@{class} stroke_width="1.6" />
        },
        "calendar-day" => html! {
            <CalendarDays class=@{class} stroke_width="1.6" />
        },
        "clock" => html! {
            <Clock class=@{class} stroke_width="1.6" />
        },
        "repeat" => html! {
            <Repeat class=@{class} stroke_width="1.6" />
        },
        "location" => html! {
            <MapPin class=@{class} stroke_width="1.6" />
        },
        "bold" => html! {
            <Bold class=@{class} stroke_width="1.6" />
        },
        "italic" => html! {
            <Italic class=@{class} stroke_width="1.6" />
        },
        "list" => html! {
            <List class=@{class} stroke_width="1.6" />
        },
        "list-ordered" => html! {
            <ListOrdered class=@{class} stroke_width="1.6" />
        },
        "arrow-right" => html! {
            <ArrowRight class=@{class} stroke_width="1.6" />
        },
        "bell" => html! {
            <Bell class=@{class} stroke_width="1.6" />
        },
        "apps" => html! {
            <Gem class=@{class} stroke_width="1.6" />
        },
        "teams" => html! {
            <Users class=@{class} stroke_width="1.6" />
        },
        "panel" => html! {
            <PanelsLeftBottom class=@{class} stroke_width="1.6" />
        },
        "chevron-left" => html! {
            <ChevronLeft class=@{class} stroke_width="1.6" />
        },
        "chevron-right" => html! {
            <ChevronRight class=@{class} stroke_width="1.6" />
        },
        "chevron-down" => html! {
            <ChevronDown class=@{class} stroke_width="1.6" />
        },
        "chevron-up" => html! {
            <ChevronUp class=@{class} stroke_width="1.6" />
        },
        "search" => html! {
            <Search class=@{class} stroke_width="1.6" />
        },
        "more" => html! {
            <Ellipsis class=@{class} stroke_width="1.6" />
        },
        "warning" => html! {
            <svg
                class=@{class}
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.6"
                stroke-linecap="round"
                stroke-linejoin="round"
                aria-hidden="true"
            >
                <path
                    d="M10.3 3.9 2.6 18a2 2 0 0 0 1.7 3h15.4a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z"
                />
                <path d="M12 8v5m0 4h.01" />
            </svg>
        },
        "sign-out" => html! {
            <svg
                class=@{class}
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.6"
                stroke-linecap="round"
                stroke-linejoin="round"
                aria-hidden="true"
            >
                <path d="M10 17v3H4V4h6v3m4 1 4 4-4 4m-6-4h10" />
            </svg>
        },
        "minimize" => html! {
            <Minus class=@{class} stroke_width="1.6" />
        },
        "maximize" => html! {
            <Square class=@{class} stroke_width="1.6" />
        },
        "close" => html! {
            <X class=@{class} stroke_width="1.6" />
        },
        "compose" => html! {
            <SquarePen class=@{class} stroke_width="1.6" />
        },
        "edit" => html! {
            <Pencil class=@{class} stroke_width="1.6" />
        },
        "emoji" => html! {
            <FaceSlightlySmiling class=@{class} stroke_width="1.6" />
        },
        "attach" => html! {
            <Paperclip class=@{class} stroke_width="1.6" />
        },
        "image" => html! {
            <Image class=@{class} stroke_width="1.6" />
        },
        "format" => html! {
            <Type class=@{class} stroke_width="1.6" />
        },
        "plus" => html! {
            <Plus class=@{class} stroke_width="1.6" />
        },
        "send" => html! {
            <Send class=@{class} stroke_width="1.6" />
        },
        "settings" => html! {
            <Settings class=@{class} stroke_width="1.6" />
        },
        "filter" => html! {
            <ListFilter class=@{class} stroke_width="1.6" />
        },
        "check-circle" => html! {
            <CircleCheck class=@{class} stroke_width="1.6" />
        },
        "check" => html! {
            <Check class=@{class} stroke_width="1.6" />
        },
        "person-add" => html! {
            <UserPlus class=@{class} stroke_width="1.6" />
        },
        "link" => html! {
            <Link class=@{class} stroke_width="1.6" />
        },
        "hash" => html! {
            <Hash class=@{class} stroke_width="1.6" />
        },
        "copy" => html! {
            <Copy class=@{class} stroke_width="1.6" />
        },
        "trash" => html! {
            <Trash2 class=@{class} stroke_width="1.6" />
        },
        "speaker" => html! {
            <Volume2 class=@{class} stroke_width="1.6" />
        },
        "mic" => html! {
            <Mic class=@{class} stroke_width="1.6" />
        },
        "mic-off" => html! {
            <MicOff class=@{class} stroke_width="1.6" />
        },
        "video-off" => html! {
            <VideoOff class=@{class} stroke_width="1.6" />
        },
        "sparkle" => html! {
            <Sparkles class=@{class} stroke_width="1.6" />
        },
        "shield" => html! {
            <Shield class=@{class} stroke_width="1.6" />
        },
        "hand" => html! {
            <Hand class=@{class} stroke_width="1.6" />
        },
        "share-screen" => html! {
            <ScreenShare class=@{class} stroke_width="1.6" />
        },
        "leave" => html! {
            <Phone class=@{leave_class.as_str()} stroke_width="1.6" />
        },
        "grid" => html! {
            <Grid2x2 class=@{class} stroke_width="1.6" />
        },
        "record" => html! {
            <CircleDot class=@{class} stroke_width="1.6" />
        },
        "upload" => html! {
            <Upload class=@{class} stroke_width="1.6" />
        },
        "bell-off" => html! {
            <BellOff class=@{class} stroke_width="1.6" />
        },
        "eye" => html! {
            <Eye class=@{class} stroke_width="1.6" />
        },
        "mail" => html! {
            <Mail class=@{class} stroke_width="1.6" />
        },
        "at" => html! {
            <AtSign class=@{class} stroke_width="1.6" />
        },
        "thumbs-up" => html! {
            <ThumbsUp class=@{class} stroke_width="1.6" />
        },
        "play" => html! {
            <Play class=@{class} stroke_width="1.6" />
        },
        "cloud" => html! {
            <Cloud class=@{class} stroke_width="1.6" />
        },
        "file" => html! {
            <File class=@{class} stroke_width="1.6" />
        },
        "paw" => html! {
            <PawPrint class=@{class} stroke_width="1.6" />
        },
        "pizza" => html! {
            <Pizza class=@{class} stroke_width="1.6" />
        },
        "volleyball" => html! {
            <Volleyball class=@{class} stroke_width="1.6" />
        },
        "file-text" => html! {
            <FileText class=@{class} stroke_width="1.6" />
        },
        "folder" => html! {
            <Folder class=@{class} stroke_width="1.6" />
        },
        "folder-plus" => html! {
            <FolderPlus class=@{class} stroke_width="1.6" />
        },
        "arrow-up" => html! {
            <ArrowUp class=@{class} stroke_width="1.6" />
        },
        "arrow-down" => html! {
            <ArrowDown class=@{class} stroke_width="1.6" />
        },
        _ => Html::empty(),
    }
}
