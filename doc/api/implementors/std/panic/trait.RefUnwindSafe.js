(function() {var implementors = {};
implementors["entity_system"] = [{"text":"impl RefUnwindSafe for Entity","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for EntityAllocator","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; RefUnwindSafe for EntityAllocatorIterator&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl&lt;EntityManagerComponentType&gt; RefUnwindSafe for EntityManager&lt;EntityManagerComponentType&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;EntityManagerComponentType: RefUnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;'a, EntityManagerComponentType&gt; !RefUnwindSafe for EntityIterator&lt;'a, EntityManagerComponentType&gt;","synthetic":true,"types":[]},{"text":"impl&lt;EntityManagerComponentType&gt; !RefUnwindSafe for Query&lt;EntityManagerComponentType&gt;","synthetic":true,"types":[]},{"text":"impl&lt;DispatcherType, EventAdapters, EventHandlerType, EventType&gt; !RefUnwindSafe for Connection&lt;DispatcherType, EventAdapters, EventHandlerType, EventType&gt;","synthetic":true,"types":[]},{"text":"impl&lt;EventType&gt; !RefUnwindSafe for Adapter&lt;EventType&gt;","synthetic":true,"types":[]},{"text":"impl&lt;EventAdapters&gt; !RefUnwindSafe for EventDispatcher&lt;EventAdapters&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; RefUnwindSafe for BasicVecStorage&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: RefUnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl RefUnwindSafe for RefreshPeriod","synthetic":true,"types":[]},{"text":"impl !RefUnwindSafe for SystemManager","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()