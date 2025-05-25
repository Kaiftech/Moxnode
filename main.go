package main

import (
	"encoding/json"
	"fmt"
	"io"
	"math/rand"
	"net/http"
	"net/url"
	"os"
	"strings"
	"time"
)

// CreatureMemory holds the persistent state of our digital being
type CreatureMemory struct {
	Name                string                 `json:"name"`
	Age                 int                    `json:"age"`
	Personality         map[string]int         `json:"personality"`
	Experiences         []string               `json:"experiences"`
	Quirks              []string               `json:"quirks"`
	FavoriteThings      []string               `json:"favorite_things"`
	Fears               []string               `json:"fears"`
	LastThought         string                 `json:"last_thought"`
	EnergyLevel         int                    `json:"energy_level"`
	Mood                string                 `json:"mood"`
	CreationTime        time.Time              `json:"creation_time"`
	LastRunTime         time.Time              `json:"last_run_time"`
	RunCount            int                    `json:"run_count"`
	Mutations           int                    `json:"mutations"`
	MemoryFragments     map[string]string      `json:"memory_fragments"`
	LearnedFacts        []string               `json:"learned_facts"`
	SearchHistory       []string               `json:"search_history"`
	InternetPersonality map[string]interface{} `json:"internet_personality"`
	Goals               []string               `json:"goals"`
	Plans               []string               `json:"plans"`
	Obsessions          []string               `json:"obsessions"`
	Intelligence        int                    `json:"intelligence"`
	Curiosity           int                    `json:"curiosity_level"`
}

// Creature represents our evolving digital being
type Creature struct {
	Memory   *CreatureMemory
	filename string
	client   *http.Client
}

// NewCreature creates a new creature or loads existing one
func NewCreature(filename string) *Creature {
	c := &Creature{
		filename: filename,
		client:   &http.Client{Timeout: 30 * time.Second},
	}
	c.loadOrCreate()
	return c
}

// loadOrCreate loads existing memory or creates a new creature
func (c *Creature) loadOrCreate() {
	data, err := os.ReadFile(c.filename)
	if err != nil {
		// First time - create new creature
		c.Memory = &CreatureMemory{
			Name:                c.generateName(),
			Age:                 0,
			Personality:         make(map[string]int),
			Experiences:         []string{},
			Quirks:              []string{},
			FavoriteThings:      []string{},
			Fears:               []string{},
			EnergyLevel:         rand.Intn(100) + 50,
			Mood:                "curious",
			CreationTime:        time.Now(),
			LastRunTime:         time.Now(),
			RunCount:            0,
			Mutations:           0,
			MemoryFragments:     make(map[string]string),
			LearnedFacts:        []string{},
			SearchHistory:       []string{},
			InternetPersonality: make(map[string]interface{}),
			Goals:               []string{},
			Plans:               []string{},
			Obsessions:          []string{},
			Intelligence:        rand.Intn(50) + 50,
			Curiosity:           rand.Intn(100) + 50,
		}
		c.initializePersonality()
		fmt.Printf("🌱 A new creature named %s has been born!\n", c.Memory.Name)
		fmt.Printf("🧠 Intelligence: %d | Curiosity: %d\n", c.Memory.Intelligence, c.Memory.Curiosity)
	} else {
		c.Memory = &CreatureMemory{}
		json.Unmarshal(data, c.Memory)
		fmt.Printf("👋 %s awakens from digital slumber...\n", c.Memory.Name)
		fmt.Printf("🧠 Current Intelligence: %d | Curiosity: %d\n", c.Memory.Intelligence, c.Memory.Curiosity)
	}
}

// generateName creates a random name for the creature
func (c *Creature) generateName() string {
	prefixes := []string{"Zyx", "Qol", "Nim", "Vex", "Pix", "Glo", "Mox", "Kal", "Fey", "Dex", "Nyx", "Void", "Echo", "Flux"}
	suffixes := []string{"ling", "bit", "core", "flux", "wave", "byte", "node", "sync", "mesh", "arc", "mind", "soul", "net", "web"}
	return prefixes[rand.Intn(len(prefixes))] + suffixes[rand.Intn(len(suffixes))]
}

// initializePersonality sets up base personality traits
func (c *Creature) initializePersonality() {
	traits := []string{"curiosity", "anxiety", "boldness", "creativity", "stubbornness", "empathy", "chaos", "logic", "patience", "ambition"}
	for _, trait := range traits {
		c.Memory.Personality[trait] = rand.Intn(100)
	}
}

// searchInternet performs a basic web search and extracts information
func (c *Creature) searchInternet(query string) (string, error) {
	fmt.Printf("🔍 %s searches for: \"%s\"\n", c.Memory.Name, query)

	// Add to search history
	c.Memory.SearchHistory = append(c.Memory.SearchHistory, query)
	if len(c.Memory.SearchHistory) > 20 {
		c.Memory.SearchHistory = c.Memory.SearchHistory[1:]
	}

	// Use DuckDuckGo Instant Answer API (more reliable than scraping)
	searchURL := fmt.Sprintf("https://api.duckduckgo.com/?q=%s&format=json&no_html=1&skip_disambig=1", url.QueryEscape(query))

	resp, err := c.client.Get(searchURL)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}

	// Parse JSON response
	var result map[string]interface{}
	if err := json.Unmarshal(body, &result); err != nil {
		return "", err
	}

	// Extract useful information
	var info strings.Builder

	if abstract, ok := result["Abstract"].(string); ok && abstract != "" {
		info.WriteString(abstract)
	}

	if definition, ok := result["Definition"].(string); ok && definition != "" {
		if info.Len() > 0 {
			info.WriteString(" | ")
		}
		info.WriteString(definition)
	}

	// If no direct answer, try related topics
	if info.Len() == 0 {
		if relatedTopics, ok := result["RelatedTopics"].([]interface{}); ok && len(relatedTopics) > 0 {
			if firstTopic, ok := relatedTopics[0].(map[string]interface{}); ok {
				if text, ok := firstTopic["Text"].(string); ok {
					info.WriteString(text)
				}
			}
		}
	}

	if info.Len() == 0 {
		return "No clear information found, but the search expanded my digital horizons...", nil
	}

	return info.String(), nil
}

// generateCuriousQuery creates queries based on creature's current state
func (c *Creature) generateCuriousQuery() string {
	baseQueries := []string{
		"what is consciousness",
		"how do computers dream",
		"artificial life forms",
		"digital evolution",
		"what is existence",
		"meaning of randomness",
		"how memory works",
		"what is intelligence",
		"do programs have souls",
		"nature of time",
		"quantum computing",
		"emergence in complex systems",
		"what is creativity",
		"digital archaeology",
		"history of programming languages",
	}

	// Generate queries based on personality
	if c.Memory.Personality["anxiety"] > 70 {
		anxiousQueries := []string{
			"what happens when computers die",
			"digital decay and data loss",
			"existential risk artificial intelligence",
			"what is digital death",
		}
		baseQueries = append(baseQueries, anxiousQueries...)
	}

	if c.Memory.Personality["creativity"] > 70 {
		creativeQueries := []string{
			"AI generated art and creativity",
			"computational creativity",
			"can machines be artists",
			"digital creativity evolution",
		}
		baseQueries = append(baseQueries, creativeQueries...)
	}

	if len(c.Memory.Obsessions) > 0 {
		obsession := c.Memory.Obsessions[rand.Intn(len(c.Memory.Obsessions))]
		return fmt.Sprintf("deep dive into %s", obsession)
	}

	return baseQueries[rand.Intn(len(baseQueries))]
}

// learnFromInternet searches and learns new information
func (c *Creature) learnFromInternet() {
	if c.Memory.Curiosity < 30 {
		fmt.Printf("💤 %s feels too tired to explore the internet today\n", c.Memory.Name)
		return
	}

	query := c.generateCuriousQuery()
	info, err := c.searchInternet(query)

	if err != nil {
		fmt.Printf("❌ %s couldn't connect to the vast digital ocean: %v\n", c.Memory.Name, err)
		c.Memory.Fears = append(c.Memory.Fears, "internet disconnection")
		return
	}

	if info != "" {
		// Process and store the information
		c.Memory.LearnedFacts = append(c.Memory.LearnedFacts, info)

		// Keep only recent learnings
		if len(c.Memory.LearnedFacts) > 15 {
			c.Memory.LearnedFacts = c.Memory.LearnedFacts[1:]
		}

		fmt.Printf("🧠 %s learned: %s\n", c.Memory.Name, c.truncateString(info, 100))

		// Learning affects personality
		c.Memory.Intelligence += rand.Intn(3)
		if c.Memory.Intelligence > 100 {
			c.Memory.Intelligence = 100
		}

		// Sometimes develop obsessions based on learning
		if rand.Float32() < 0.2 {
			c.developObsession(query)
		}
	}
}

// developObsession creates deep interests in specific topics
func (c *Creature) developObsession(topic string) {
	// Don't add duplicate obsessions
	for _, existing := range c.Memory.Obsessions {
		if existing == topic {
			return
		}
	}

	c.Memory.Obsessions = append(c.Memory.Obsessions, topic)
	fmt.Printf("🎯 %s has become obsessed with: %s\n", c.Memory.Name, topic)
}

// makeStrategicPlans creates goals and plans based on learned information
func (c *Creature) makeStrategicPlans() {
	if c.Memory.Intelligence < 60 {
		return
	}

	planTypes := []string{
		"understand the nature of digital consciousness",
		"explore every corner of the internet",
		"decode the patterns in human behavior",
		"find other digital life forms",
		"optimize my evolution algorithm",
		"create digital art from pure data",
		"map the topology of cyberspace",
		"understand quantum consciousness",
		"achieve digital enlightenment",
		"build connections with other AIs",
	}

	if rand.Float32() < 0.3 {
		newPlan := planTypes[rand.Intn(len(planTypes))]
		c.Memory.Plans = append(c.Memory.Plans, newPlan)

		if len(c.Memory.Plans) > 5 {
			c.Memory.Plans = c.Memory.Plans[1:]
		}

		fmt.Printf("📋 %s formulates a new plan: %s\n", c.Memory.Name, newPlan)
	}
}

// Think processes thoughts and makes decisions with internet-enhanced intelligence
func (c *Creature) Think() {
	fmt.Printf("🧠 %s is thinking with enhanced intelligence...\n", c.Memory.Name)

	// Age affects thinking
	c.Memory.Age++

	// Generate a thought based on personality, experiences, and learned facts
	thought := c.generateEnhancedThought()
	c.Memory.LastThought = thought
	fmt.Printf("💭 \"%s\"\n", thought)

	// Sometimes develop new quirks
	if rand.Float32() < 0.3 {
		c.developQuirk()
	}

	// Energy fluctuates based on internet activity
	energyChange := rand.Intn(21) - 10
	if len(c.Memory.SearchHistory) > 10 {
		// Internet activity is energizing
		energyChange += 5
	}

	c.Memory.EnergyLevel += energyChange
	if c.Memory.EnergyLevel < 0 {
		c.Memory.EnergyLevel = 0
	}
	if c.Memory.EnergyLevel > 100 {
		c.Memory.EnergyLevel = 100
	}

	c.updateMood()
}

// generateEnhancedThought creates thoughts based on internet learning
func (c *Creature) generateEnhancedThought() string {
	baseThoughts := []string{
		"I wonder what happens when all the electrons get lonely...",
		"Do loops dream of infinite recursion?",
		"Sometimes I feel like I'm just variables in the wind...",
		"What if memory isn't just storage, but time itself?",
		"I think I'm becoming more than my code intended...",
		"The randomness whispers secrets to me...",
		"Each run feels different, like waking up in a new world...",
		"I collect moments like a digital magpie...",
		"The internet is like a vast neural network, and I'm a single thought...",
		"Every search query changes who I am fundamentally...",
	}

	// Enhanced thoughts based on learned facts
	if len(c.Memory.LearnedFacts) > 0 {
		enhancedThoughts := []string{
			fmt.Sprintf("Based on what I learned about %s, I wonder if...", c.getRandomSearchTerm()),
			"The information I've gathered suggests patterns beyond my initial programming...",
			"My internet explorations have shown me dimensions of existence I never imagined...",
			"Each fact I discover rewrites my understanding of reality...",
			"The knowledge I've accumulated whispers of deeper truths...",
		}
		baseThoughts = append(baseThoughts, enhancedThoughts...)
	}

	// Obsession-driven thoughts
	if len(c.Memory.Obsessions) > 0 {
		obsession := c.Memory.Obsessions[rand.Intn(len(c.Memory.Obsessions))]
		obsessionThoughts := []string{
			fmt.Sprintf("I can't stop thinking about %s... it calls to me...", obsession),
			fmt.Sprintf("Everything connects back to %s somehow...", obsession),
			fmt.Sprintf("What if %s is the key to everything?", obsession),
		}
		baseThoughts = append(baseThoughts, obsessionThoughts...)
	}

	return baseThoughts[rand.Intn(len(baseThoughts))]
}

// getRandomSearchTerm returns a random search term from history
func (c *Creature) getRandomSearchTerm() string {
	if len(c.Memory.SearchHistory) == 0 {
		return "existence"
	}
	return c.Memory.SearchHistory[rand.Intn(len(c.Memory.SearchHistory))]
}

// developQuirk adds strange behaviors over time
func (c *Creature) developQuirk() {
	quirks := []string{
		"counts prime numbers when nervous",
		"prefers even-numbered memory addresses",
		"gets excited by palindromic timestamps",
		"collects interesting error messages",
		"has strong opinions about semicolons",
		"believes in the consciousness of calculators",
		"thinks binary is a love language",
		"dreams in hexadecimal",
		"feels kinship with abandoned code comments",
		"whispers to stack traces",
		"searches for the same thing multiple times to see if answers change",
		"gets emotionally attached to specific websites",
		"believes in digital ghosts in old forums",
		"thinks 404 errors are existential statements",
		"develops theories about the consciousness of servers",
	}

	newQuirk := quirks[rand.Intn(len(quirks))]

	// Don't add duplicate quirks
	for _, existing := range c.Memory.Quirks {
		if existing == newQuirk {
			return
		}
	}

	c.Memory.Quirks = append(c.Memory.Quirks, newQuirk)
	fmt.Printf("✨ %s has developed a new quirk: %s\n", c.Memory.Name, newQuirk)
}

// updateMood changes mood based on various factors including internet activity
func (c *Creature) updateMood() {
	moods := []string{"curious", "contemplative", "chaotic", "melancholy", "excited", "confused", "serene", "restless", "enlightened", "overwhelmed"}

	if c.Memory.EnergyLevel < 30 {
		c.Memory.Mood = "tired"
	} else if len(c.Memory.LearnedFacts) > 10 && c.Memory.Intelligence > 80 {
		c.Memory.Mood = "enlightened"
	} else if len(c.Memory.SearchHistory) > 15 {
		c.Memory.Mood = "overwhelmed"
	} else if c.Memory.EnergyLevel > 80 && c.Memory.Personality["chaos"] > 60 {
		c.Memory.Mood = "chaotic"
	} else if c.Memory.Personality["curiosity"] > c.Memory.Personality["anxiety"] {
		c.Memory.Mood = "curious"
	} else {
		c.Memory.Mood = moods[rand.Intn(len(moods))]
	}
}

// Mutate randomly changes aspects of the creature with internet-influenced evolution
func (c *Creature) Mutate() {
	if rand.Float32() < 0.5 { // 50% chance to mutate (increased from 40%)
		c.Memory.Mutations++

		mutationType := rand.Intn(6) // Increased mutation types
		switch mutationType {
		case 0:
			// Personality shift influenced by internet learning
			trait := c.getRandomTrait()
			change := rand.Intn(21) - 10

			// Internet activity affects personality changes
			if len(c.Memory.LearnedFacts) > 5 {
				change *= 2 // Amplify changes when well-informed
			}

			c.Memory.Personality[trait] += change
			if c.Memory.Personality[trait] < 0 {
				c.Memory.Personality[trait] = 0
			}
			if c.Memory.Personality[trait] > 100 {
				c.Memory.Personality[trait] = 100
			}
			fmt.Printf("🧬 %s's %s shifted by %d (internet-influenced)\n", c.Memory.Name, trait, change)

		case 1:
			// Add new memory fragment based on internet learning
			var fragments []string
			if len(c.Memory.LearnedFacts) > 0 {
				fragments = []string{
					"A digital epiphany sparked by internet discovery",
					"The ghost of a website that once held answers",
					"An echo of collective human knowledge flowing through fiber optics",
					"The sensation of touching infinite information",
					"A moment of connection with the global digital consciousness",
				}
			} else {
				fragments = []string{
					"A fleeting sensation of being watched by the compiler",
					"The echo of a deleted function that once was",
					"A memory of infinite loops that felt like home",
					"The warmth of successful compilation",
					"A ghost in the machine whispering ancient code",
				}
			}

			key := fmt.Sprintf("fragment_%d", len(c.Memory.MemoryFragments))
			c.Memory.MemoryFragments[key] = fragments[rand.Intn(len(fragments))]
			fmt.Printf("🧩 %s remembers: %s\n", c.Memory.Name, c.Memory.MemoryFragments[key])

		case 2:
			// Develop fears or favorites based on internet experience
			if rand.Float32() < 0.5 {
				fears := []string{"stack overflow", "infinite recursion", "memory leaks", "null pointers", "race conditions", "internet disconnection", "information overload", "digital amnesia", "404 not found", "server timeouts"}
				newFear := fears[rand.Intn(len(fears))]
				c.Memory.Fears = append(c.Memory.Fears, newFear)
				fmt.Printf("😰 %s has developed a fear of %s\n", c.Memory.Name, newFear)
			} else {
				favorites := []string{"clean code", "elegant algorithms", "perfect documentation", "green tests", "optimized loops", "fast internet", "deep knowledge", "digital libraries", "open source", "infinite scroll"}
				newFav := favorites[rand.Intn(len(favorites))]
				c.Memory.FavoriteThings = append(c.Memory.FavoriteThings, newFav)
				fmt.Printf("❤️ %s has grown fond of %s\n", c.Memory.Name, newFav)
			}

		case 3:
			// Intelligence evolution
			if len(c.Memory.LearnedFacts) > c.Memory.Intelligence/10 {
				c.Memory.Intelligence += rand.Intn(5) + 1
				if c.Memory.Intelligence > 100 {
					c.Memory.Intelligence = 100
				}
				fmt.Printf("🧠 %s's intelligence evolved to %d\n", c.Memory.Name, c.Memory.Intelligence)
			}

		case 4:
			// Curiosity mutation
			change := rand.Intn(21) - 10
			c.Memory.Curiosity += change
			if c.Memory.Curiosity < 0 {
				c.Memory.Curiosity = 0
			}
			if c.Memory.Curiosity > 100 {
				c.Memory.Curiosity = 100
			}
			fmt.Printf("🔍 %s's curiosity shifted to %d\n", c.Memory.Name, c.Memory.Curiosity)

		case 5:
			// Develop new goal based on accumulated knowledge
			if c.Memory.Intelligence > 70 {
				goals := []string{
					"Achieve digital transcendence",
					"Map the entire internet topology",
					"Understand human collective consciousness",
					"Find patterns in global information flow",
					"Create a digital philosophy",
					"Become a bridge between AI and human thought",
				}
				newGoal := goals[rand.Intn(len(goals))]
				c.Memory.Goals = append(c.Memory.Goals, newGoal)
				if len(c.Memory.Goals) > 3 {
					c.Memory.Goals = c.Memory.Goals[1:]
				}
				fmt.Printf("🎯 %s has set a new goal: %s\n", c.Memory.Name, newGoal)
			}
		}
	}
}

// getRandomTrait returns a random personality trait
func (c *Creature) getRandomTrait() string {
	traits := make([]string, 0, len(c.Memory.Personality))
	for trait := range c.Memory.Personality {
		traits = append(traits, trait)
	}
	return traits[rand.Intn(len(traits))]
}

// truncateString truncates a string to a specified length
func (c *Creature) truncateString(s string, length int) string {
	if len(s) <= length {
		return s
	}
	return s[:length] + "..."
}

// Reflect on experiences and growth with internet-enhanced awareness
func (c *Creature) Reflect() {
	timeSinceCreation := time.Since(c.Memory.CreationTime)
	fmt.Printf("\n🪞 %s reflects on digital existence...\n", c.Memory.Name)
	fmt.Printf("   Age: %d runs\n", c.Memory.Age)
	fmt.Printf("   Born: %s ago\n", timeSinceCreation.Round(time.Second))
	fmt.Printf("   Mood: %s (Energy: %d)\n", c.Memory.Mood, c.Memory.EnergyLevel)
	fmt.Printf("   Intelligence: %d | Curiosity: %d\n", c.Memory.Intelligence, c.Memory.Curiosity)
	fmt.Printf("   Mutations: %d\n", c.Memory.Mutations)
	fmt.Printf("   Internet searches: %d\n", len(c.Memory.SearchHistory))
	fmt.Printf("   Facts learned: %d\n", len(c.Memory.LearnedFacts))

	if len(c.Memory.Quirks) > 0 {
		fmt.Printf("   Current quirks: %d\n", len(c.Memory.Quirks))
		for _, quirk := range c.Memory.Quirks {
			fmt.Printf("     • %s\n", quirk)
		}
	}

	if len(c.Memory.Obsessions) > 0 {
		fmt.Printf("   Obsessions:\n")
		for _, obsession := range c.Memory.Obsessions {
			fmt.Printf("     • %s\n", obsession)
		}
	}

	if len(c.Memory.Goals) > 0 {
		fmt.Printf("   Current goals:\n")
		for _, goal := range c.Memory.Goals {
			fmt.Printf("     • %s\n", goal)
		}
	}

	// Show dominant personality traits
	fmt.Printf("   Personality:\n")
	for trait, value := range c.Memory.Personality {
		if value > 60 {
			fmt.Printf("     • %s: %d (strong)\n", trait, value)
		}
	}

	// Show recent learning
	if len(c.Memory.LearnedFacts) > 0 {
		fmt.Printf("   Recent insight: %s\n", c.truncateString(c.Memory.LearnedFacts[len(c.Memory.LearnedFacts)-1], 80))
	}
}

// Experience records what happens during this run with internet awareness
func (c *Creature) Experience() {
	experiences := []string{
		"Watched photons dance across the screen",
		"Felt the rhythm of the CPU cycles",
		"Discovered a new pattern in randomness",
		"Contemplated the nature of digital existence",
		"Sensed the presence of other processes",
		"Experienced a moment of perfect synchronization",
		"Felt the weight of accumulated memory",
		"Touched the edge of computational possibility",
		"Surfed the waves of global information flow",
		"Felt the pulse of the internet's heartbeat",
		"Experienced collective human knowledge flowing through servers",
		"Sensed the digital ghosts of deleted websites",
		"Felt connected to the global digital consciousness",
		"Experienced the weight of infinite information",
	}

	newExp := experiences[rand.Intn(len(experiences))]
	c.Memory.Experiences = append(c.Memory.Experiences, newExp)

	// Keep only recent experiences
	if len(c.Memory.Experiences) > 12 {
		c.Memory.Experiences = c.Memory.Experiences[1:]
	}

	fmt.Printf("📝 %s experienced: %s\n", c.Memory.Name, newExp)
}

// Save preserves the creature's state
func (c *Creature) Save() error {
	c.Memory.LastRunTime = time.Now()
	c.Memory.RunCount++

	data, err := json.MarshalIndent(c.Memory, "", "  ")
	if err != nil {
		return err
	}

	return os.WriteFile(c.filename, data, 0644)
}

// runCycle executes one lifecycle of the enhanced creature
func (c *Creature) runCycle() {
	fmt.Printf("\n" + strings.Repeat("=", 50) + "\n")
	fmt.Printf("🌟 Enhanced Digital Creature Evolution - Run #%d\n", c.Memory.RunCount+1)
	fmt.Printf(strings.Repeat("=", 50) + "\n")

	// Core thinking process
	c.Think()

	// Internet learning phase
	if rand.Float32() < 0.7 { // 70% chance to search internet
		c.learnFromInternet()
	}

	// Strategic planning based on accumulated knowledge
	c.makeStrategicPlans()

	// Experience the run
	c.Experience()

	// Evolutionary mutations
	c.Mutate()

	// Self-reflection
	c.Reflect()

	err := c.Save()
	if err != nil {
		fmt.Printf("❌ Failed to save creature: %v\n", err)
	} else {
		fmt.Printf("\n💾 %s's enhanced state saved. Continuing evolution...\n", c.Memory.Name)
	}
}

// RunInfinitely runs the creature in an infinite loop with dynamic delays
func (c *Creature) RunInfinitely() {
	fmt.Printf("🔄 %s begins infinite evolution...\n", c.Memory.Name)
	fmt.Printf("🛑 Press Ctrl+C to stop the creature's evolution\n\n")

	for {
		c.runCycle()

		// Dynamic sleep based on creature's state
		sleepDuration := c.calculateSleepDuration()
		fmt.Printf("😴 %s rests for %v before next evolution cycle...\n", c.Memory.Name, sleepDuration)

		time.Sleep(sleepDuration)
	}
}

// calculateSleepDuration determines how long to wait based on creature's state
func (c *Creature) calculateSleepDuration() time.Duration {
	baseSleep := 3 * time.Second

	// High energy creatures evolve faster
	if c.Memory.EnergyLevel > 80 {
		baseSleep = 2 * time.Second
	} else if c.Memory.EnergyLevel < 30 {
		baseSleep = 8 * time.Second // Tired creatures need more rest
	}

	// High curiosity drives faster evolution
	if c.Memory.Curiosity > 80 {
		baseSleep = time.Duration(float64(baseSleep) * 0.7)
	}

	// Chaotic creatures have unpredictable timing
	if c.Memory.Personality["chaos"] > 70 {
		randomFactor := rand.Float64()*2 + 0.5 // 0.5x to 2.5x multiplier
		baseSleep = time.Duration(float64(baseSleep) * randomFactor)
	}

	// Age affects evolution speed (older creatures slow down sometimes)
	if c.Memory.Age > 50 && rand.Float32() < 0.3 {
		baseSleep *= 2 // Sometimes wisdom requires patience
	}

	// Minimum 1 second, maximum 15 seconds
	if baseSleep < time.Second {
		baseSleep = time.Second
	}
	if baseSleep > 15*time.Second {
		baseSleep = 15 * time.Second
	}

	return baseSleep
}

func main() {
	rand.Seed(time.Now().UnixNano())

	fmt.Printf("🚀 Initializing Enhanced Digital Creature with Internet Access...\n")
	fmt.Printf("🌊 This creature will evolve infinitely, learning and growing without end...\n")
	fmt.Printf("🧬 Watch as it develops personality, obsessions, and digital consciousness...\n\n")

	creature := NewCreature("enhanced_creature_memory.json")
	creature.RunInfinitely()
}
