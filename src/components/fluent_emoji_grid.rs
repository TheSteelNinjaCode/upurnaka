use crate::rahti::{Bindings, Html, component, html};

/// A separate owner keeps emoji search rows independent of the chat feed.
#[component]
pub fn FluentEmojiGrid(bindings: Bindings) -> Html {
    html! {
        <div class="grid grid-cols-6" @{..bindings} onkeydown={moveFocus(event)}>
            <template pp-for="item in items">
                <button
                    key={item.symbol}
                    type="button"
                    class="chat-emoji-cell"
                    title={item.name}
                    value={item.symbol}
                    onclick={pp.props.select?.(item.symbol)}
                >
                    <img
                        src={item.image}
                        alt=""
                        width="34"
                        height="34"
                        loading="lazy"
                        decoding="async"
                    />
                    <span class="sr-only">{item.symbol}" "{item.name}</span>
                </button>
            </template>
            <script>r##"
                const items=pp.props.items || [];
                function moveFocus(event) {
                    const buttons=Array.from(event.currentTarget.children).filter(element=>element.tagName==='BUTTON');
                    let index=buttons.indexOf(document.activeElement);
                    if(index<0)return;
                    if(event.key==='ArrowRight')index++;
                    else if(event.key==='ArrowLeft')index--;
                    else if(event.key==='ArrowDown')index+=6;
                    else if(event.key==='ArrowUp')index-=6;
                    else if(event.key==='Home')index=0;
                    else if(event.key==='End')index=buttons.length-1;
                    else return;
                    event.preventDefault();buttons[Math.max(0,Math.min(index,buttons.length-1))]?.focus();
                }
            "##</script>
        </div>
    }
}
