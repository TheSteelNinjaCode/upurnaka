//! Application icon names backed by installed rahti-icons components.

use crate::components::rahti_icons::bell::Bell;
use crate::components::rahti_icons::calendar::Calendar;
use crate::components::rahti_icons::calendar_days::CalendarDays;
use crate::components::rahti_icons::check::Check;
use crate::components::rahti_icons::chevron_down::ChevronDown;
use crate::components::rahti_icons::chevron_left::ChevronLeft;
use crate::components::rahti_icons::chevron_right::ChevronRight;
use crate::components::rahti_icons::chevron_up::ChevronUp;
use crate::components::rahti_icons::circle_check::CircleCheck;
use crate::components::rahti_icons::contact::Contact;
use crate::components::rahti_icons::copy::Copy;
use crate::components::rahti_icons::ellipsis::Ellipsis;
use crate::components::rahti_icons::face_slightly_smiling::FaceSlightlySmiling;
use crate::components::rahti_icons::gem::Gem;
use crate::components::rahti_icons::hash::Hash;
use crate::components::rahti_icons::image::Image;
use crate::components::rahti_icons::link::Link;
use crate::components::rahti_icons::list_filter::ListFilter;
use crate::components::rahti_icons::message_circle::MessageCircle;
use crate::components::rahti_icons::mic::Mic;
use crate::components::rahti_icons::mic_off::MicOff;
use crate::components::rahti_icons::minus::Minus;
use crate::components::rahti_icons::panels_left_bottom::PanelsLeftBottom;
use crate::components::rahti_icons::paperclip::Paperclip;
use crate::components::rahti_icons::plus::Plus;
use crate::components::rahti_icons::search::Search;
use crate::components::rahti_icons::send::Send;
use crate::components::rahti_icons::settings::Settings;
use crate::components::rahti_icons::sparkles::Sparkles;
use crate::components::rahti_icons::square::Square;
use crate::components::rahti_icons::square_pen::SquarePen;
use crate::components::rahti_icons::trash_2::Trash2;
use crate::components::rahti_icons::type_icon::Type;
use crate::components::rahti_icons::user_plus::UserPlus;
use crate::components::rahti_icons::users::Users;
use crate::components::rahti_icons::video::Video;
use crate::components::rahti_icons::video_off::VideoOff;
use crate::components::rahti_icons::x::X;
use crate::rahti::{Html, component, html};

#[component]
pub fn Icon(name: &str, class: &str) -> Html {
    match name {
        "chat" => html! {
            <MessageCircle class=@{class} stroke_width="1.6" />
        },
        "video" => html! {
            <Video class=@{class} stroke_width="1.6" />
        },
        "contacts" => html! {
            <Contact class=@{class} stroke_width="1.6" />
        },
        "copilot" => html! {
            <Sparkles class=@{class} stroke_width="1.6" />
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
        _ => Html::empty(),
    }
}
