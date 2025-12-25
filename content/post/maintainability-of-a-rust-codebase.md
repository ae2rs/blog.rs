---
title: 'how to avoid legacy hell and build maintainable systems'
published: 2025-10-04
draft: true
---

Recently I have be frustrated with the way maintainability is approached (or rather, receded from) in most complex software systems in two main ways:

- It is seen as a chore rather than a blessing, often being assimilated to the endless [refactoring meme](https://preview.redd.it/pleasestop-v0-txr7gptyv1ad1.jpeg?width=1080&crop=smart&auto=webp&s=abbfa10a91eb5c9d099c3128320fa150e7c4078c), even though it is [completely different]('!TODO: add link to self part')
- It gets painted as a time-waster that would distract the team from doing more "productive work", especially in fast-growing organizations

Before you start thinking this is just another rant from an overly strict (often german) software engineer that thinks all there is about a business is the quality of its engineering -- I promise, its not.

I actually want you to realize how sexy maintainability can be, for both engineering teams and product managers. But let's not get ahead of ourselves. First, lets go back to the basics of what we want to talk about, what maintainability is.

# What is maintainability?

Instinctively, most developpers would define maintainability with a gut-feely eponym expression like "it describes how easy a system is to maintain over time". After giving it more thought, they would probably come up with a more precise but less complete answer, describing its implementation rather than its nature.

A good example of this is how well known acronyms like KISS, DRY and variations of the like pop up whenever code quality or maintainability are discussed. Ignoring the lack of any depth and reflexion that these concepts provide, they are not bad per se. But this is really missing the forest for the trees.

Maintainability is much simpler, much more fundamental than all of that. Borrowing from the obectively incredible [book by Martin Kleppmann](https://www.oreilly.com/library/view/designing-data-intensive-applications/9781491903063/), he introduces the concept of maintainability like this:

> Over time, many different people will work on the system (engineering and operations, both maintaining current behavior and adapting the system to new use cases), and they should all be able to work on it productively.

The key word here is _*productively*_. Maintainability is simply the quality of a system in its ability to not get in the way of the people working on it. And this benefits everyone!

- Engineers themselves, as they spend less time wondering which disgusting part of the codebase they are going to have to patch for the 100th time before it all comes crashing down
- PMs, as having a maintainable system makes feature estimates more precise and predictable
- Management (and/or stakeholders), as more productivity means faster execution means [more money](https://en.meming.world/wiki/Stonks)

But maintainability comes at a cost.
Sometimes, you can't afford to take the 20% more time required to make things cleaner simply because engineering is just a small cog in the huge machines that big (or small) companies are.

Sometimes, you have to go fast, write shit code and ship a hotfix at 6am after a night of debugging. Luckily for you, that's not what we are doing today.

Ready? Let's dive in.

# Navigation: the art of finding things effectively

When we talk about coding, we don't think about how we *use* the code. We often only discuss how to *write* it. And that's a big problem.

And when I say use, I don't mean someone else using your lib. I mean using the code itself, jumping from definitions to definitions, reading the body of a function you want to use and doesn't have comments, searching for that one file where you remember seeing the struct you need, etc.

Concurrently, this is the thing we spend the most time on in a software project. Before making any changes or estimates, you research, you evaluate the current state of things or you investigate the origin of a bug. So shouldn't it be a priority to optimize for that?

Again, maintainability is defined as the capacity of a system to let its maintainers work on it productively.

But how do we make a codebase more easily navigatable?

## Locality

> The primary feature for easy maintenance is locality: Locality is that characteristic of source code that enables a programmer to understand that source by looking at only a small portion of it. 
> [Richard Gabriel](https://www.dreamsongs.com/Files/PatternsOfSoftware.pdf)


