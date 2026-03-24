---
title: Static Site Generator
date: 2026-03-24
template: templates/post.html
---
As an actual first post, I thought it would be worth writing about the code that creates this website, and my reasoning for writing the code how I have.  For anyone who is interested, the code can be found at [GitHub](https://github.com/OrangeBacon/orangebacon.github.io).

# Aims
The main aim of the code I want something that actually works.  I have had several previous attempts at writing my own website, however none of them have actually worked, I have always just had partially complete websites left abandoned.  Therefore, I want to write as little as possible code to just get this thing to work.

# Why Did I Write This
There are plenty of different ways to publish content on the internet, I'm sure that this site could have been hosted with something like WordPress, on a free hosting service, or used any other existing platform, however there are several downsides to this.

Pretty much all proprietary solutions to hosting a website require you to write the text inside their editor, or on their website.  This makes it harder to move out of their system if I want to change which company I am using.

For some of these pages there might be all kinds of custom assets, e.g. javascript, diagrams, formulas, and other things I haven't thought of yet.  It is really easy to include these within a site if it is all under my control, however it would likely be very hard to use custom code on a free WordPress site.

# Why a Static Site Generator
The main reason for choosing a static site generator is that that is all I need!  There isn't any content on here which requires any particular backend code, everything on the site (at least currently) is front-end only.

Additionally, it makes it really easy to move the site between different hosting methods, as it is as simple as copying a folder.

# What the site does do!
Since I've written about what the code doesn't do, here is what it does do:
- Parse posts written in markdown and converts them into html.  This currently uses `pulldown-cmark` as the parser, with its default html output.
- Put the parsed markdown into a template with header, footer and other html that is common between all posts.
- Creates the index.html landing page from a different template.
- Copies all other files (just CSS currently) to the output folder.

# What I want to improve
There are a lot of parts of this site that could be improved.  The styling of the site isn't great currently, there is a lot that could be improved with the CSS and how everything is laid out on the page.  This however is easy to change over time until I get something that I actually like the look of.

I expect as I write more posts, I will want more complex formatting from the markdown parser, at some point I might not want to just use the default markdown to html processing.  I will only change this at the time that I actually use the formatting though, rather than getting caught up in writing custom processing for things I'm not going to use.

It would also be good to have tags for each post, and pages to filter by tags, so you can see related posts.  However, it doesn't really make sense to add this complexity with just 2 posts on the site.

Finally, I would like to improve the hosting.  Currently, it is hosted using GitHub pages, however there are a lot of limitations with this, in particular about how easy it is to host images, as the entire site has to be within a git repo.  It would likely be pretty easy to host this site entirely on a single cheap VPS, with Cloudflare's free caching in front of the server, however I haven't done this, and likely won't until there is enough content to make it worth doing.
