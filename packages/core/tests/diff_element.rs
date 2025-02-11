use dioxus::core::Mutation::*;
use dioxus::prelude::*;
use dioxus_core::{AttributeValue, ElementId};

#[test]
fn text_diff() {
    fn app(cx: Scope) -> Element {
        let gen = cx.generation();
        cx.render(rsx!( h1 { "hello {gen}" } ))
    }

    let mut vdom = VirtualDom::new(app);
    _ = vdom.rebuild();

    vdom.mark_dirty(ScopeId::ROOT);
    assert_eq!(
        vdom.render_immediate().edits,
        [SetText { value: "hello 1", id: ElementId(2) }]
    );

    vdom.mark_dirty(ScopeId::ROOT);
    assert_eq!(
        vdom.render_immediate().edits,
        [SetText { value: "hello 2", id: ElementId(2) }]
    );

    vdom.mark_dirty(ScopeId::ROOT);
    assert_eq!(
        vdom.render_immediate().edits,
        [SetText { value: "hello 3", id: ElementId(2) }]
    );
}

#[test]
fn element_swap() {
    fn app(cx: Scope) -> Element {
        let gen = cx.generation();

        match gen % 2 {
            0 => cx.render(rsx!( h1 { "hello 1" } )),
            1 => cx.render(rsx!( h2 { "hello 2" } )),
            _ => unreachable!(),
        }
    }

    let mut vdom = VirtualDom::new(app);
    _ = vdom.rebuild();

    vdom.mark_dirty(ScopeId::ROOT);
    assert_eq!(
        vdom.render_immediate().santize().edits,
        [
            LoadTemplate { name: "template", index: 0, id: ElementId(2,) },
            ReplaceWith { id: ElementId(1,), m: 1 },
        ]
    );

    vdom.mark_dirty(ScopeId::ROOT);
    assert_eq!(
        vdom.render_immediate().santize().edits,
        [
            LoadTemplate { name: "template", index: 0, id: ElementId(1,) },
            ReplaceWith { id: ElementId(2,), m: 1 },
        ]
    );

    vdom.mark_dirty(ScopeId::ROOT);
    assert_eq!(
        vdom.render_immediate().santize().edits,
        [
            LoadTemplate { name: "template", index: 0, id: ElementId(2,) },
            ReplaceWith { id: ElementId(1,), m: 1 },
        ]
    );

    vdom.mark_dirty(ScopeId::ROOT);
    assert_eq!(
        vdom.render_immediate().santize().edits,
        [
            LoadTemplate { name: "template", index: 0, id: ElementId(1,) },
            ReplaceWith { id: ElementId(2,), m: 1 },
        ]
    );
}

#[test]
fn attribute_diff() {
    fn app(cx: Scope) -> Element {
        let gen = cx.generation();

        // attributes have to be sorted by name
        let attrs = match gen % 5 {
            0 => cx.bump().alloc([Attribute::new(
                "a",
                AttributeValue::Text("hello"),
                None,
                false,
            )]) as &[Attribute],
            1 => cx.bump().alloc([
                Attribute::new("a", AttributeValue::Text("hello"), None, false),
                Attribute::new("b", AttributeValue::Text("hello"), None, false),
                Attribute::new("c", AttributeValue::Text("hello"), None, false),
            ]) as &[Attribute],
            2 => cx.bump().alloc([
                Attribute::new("c", AttributeValue::Text("hello"), None, false),
                Attribute::new("d", AttributeValue::Text("hello"), None, false),
                Attribute::new("e", AttributeValue::Text("hello"), None, false),
            ]) as &[Attribute],
            3 => cx.bump().alloc([Attribute::new(
                "d",
                AttributeValue::Text("world"),
                None,
                false,
            )]) as &[Attribute],
            _ => unreachable!(),
        };

        cx.render(rsx!(
            div {
                ..*attrs,
                "hello"
            }
        ))
    }

    let mut vdom = VirtualDom::new(app);
    _ = vdom.rebuild();

    vdom.mark_dirty(ScopeId::ROOT);
    assert_eq!(
        vdom.render_immediate().santize().edits,
        [
            SetAttribute {
                name: "b",
                value: (&AttributeValue::Text("hello",)).into(),
                id: ElementId(1,),
                ns: None,
            },
            SetAttribute {
                name: "c",
                value: (&AttributeValue::Text("hello",)).into(),
                id: ElementId(1,),
                ns: None,
            },
        ]
    );

    vdom.mark_dirty(ScopeId::ROOT);
    assert_eq!(
        vdom.render_immediate().santize().edits,
        [
            SetAttribute {
                name: "a",
                value: (&AttributeValue::None).into(),
                id: ElementId(1,),
                ns: None,
            },
            SetAttribute {
                name: "b",
                value: (&AttributeValue::None).into(),
                id: ElementId(1,),
                ns: None,
            },
            SetAttribute {
                name: "d",
                value: (&AttributeValue::Text("hello",)).into(),
                id: ElementId(1,),
                ns: None,
            },
            SetAttribute {
                name: "e",
                value: (&AttributeValue::Text("hello",)).into(),
                id: ElementId(1,),
                ns: None,
            },
        ]
    );

    vdom.mark_dirty(ScopeId::ROOT);
    assert_eq!(
        vdom.render_immediate().santize().edits,
        [
            SetAttribute {
                name: "c",
                value: (&AttributeValue::None).into(),
                id: ElementId(1,),
                ns: None,
            },
            SetAttribute {
                name: "d",
                value: (&AttributeValue::Text("world",)).into(),
                id: ElementId(1,),
                ns: None,
            },
            SetAttribute {
                name: "e",
                value: (&AttributeValue::None).into(),
                id: ElementId(1,),
                ns: None,
            },
        ]
    );
}
