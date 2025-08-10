import MDXComponents from '@theme-original/MDXComponents';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'; // Import the FontAwesomeIcon component.
import { library } from '@fortawesome/fontawesome-svg-core'; // Import the library component.
import { fab } from '@fortawesome/free-brands-svg-icons'; // Import all brands icons.
import { fas } from '@fortawesome/free-solid-svg-icons'; // Import all solid icons.

// add all icons to the library so you can use them without importing them individually
library.add(fab, fas);

export default {
    // re-use the default mapping
    ...MDXComponents,
    // make the FontAwesomeIcon component available in MDX as <icon />
    FAIcon: FontAwesomeIcon,
};
