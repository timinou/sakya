# Sakya

In typical fashion, In order to write my book, I'm first writing my writing environment. At least I'm not designing a new language this time.

## Introduction

Sakya is a cross-platform creative writing environment for the chronic procrastinator. It's open-source under the AGPL license, because I've been vibe-coding so much, I might as well release *one* thing in open-source for the time being.

It's meant to be cross-platform, but for now officially supports Wayland on Linux, and maybe soon Android. It uses Tauri+Svelte, two technologies I am not capable of writing myself.

## UX

Sakya is broadly inspired by my cursory understanding of the entity component system of video games.

1. **Entity**: These are the "things" that make up your story. You define entity **types**, and then you create entities from them. For example, an entity might be a character, a place, an important item, etc. You add properties to an entity type (e.g. the insecurities of each character, etc.). Down the line, entities will also have relations---haven't decided how to model them yet. Since I'm of the meta kind, you can also put 'concepts' or 'metaphors' as entity types as well. Entity types are project-specific, which means you get to redefine the axioms of your world on every single project.
2. **Components**: These are the places where your entities play with each other. For example, you can make a poem, a note, or an actual chapter of your story. This is especially great if you're like me and you'd rather write a non-linear story in bits because you think that might just subvert the crazy block you have around writing your stories.
3. **System**: At this point, just start writing dude.

