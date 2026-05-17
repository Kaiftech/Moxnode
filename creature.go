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

const defaultMemoryFile = "enhanced_creature_memory.json"

// CreatureMemory holds the persistent state of our digital being.
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

// Creature represents our evolving digital being.
type Creature struct {
	Memory      *CreatureMemory
	filename    string
	journalPath string
	client      *http.Client
}

// NewCreature creates a new creature or loads an existing one.
func NewCreature(filename, journalPath string) *Creature {
	c := &Creature{
		filename:    filename,
		journalPath: journalPath,
		client:      &http.Client{Timeout: 30 * time.Second},
	}
	c.loadOrCreate()
	return c
}

func (c *Creature) loadOrCreate() {
	data, err := os.ReadFile(c.filename)
	if err != nil {
		c.Memory = &CreatureMemory{
			Name:                c.generateName(),
			Personality:         make(map[string]int),
			EnergyLevel:         rand.Intn(100) + 50,
			Mood:                "curious",
			CreationTime:        time.Now(),
			LastRunTime:         time.Now(),
			MemoryFragments:     make(map[string]string),
			InternetPersonality: make(map[string]interface{}),
			Intelligence:        rand.Intn(50) + 50,
			Curiosity:           rand.Intn(100) + 50,
		}
		c.initializePersonality()
		fmt.Printf("🌱 A new creature named %s has been born!\n", c.Memory.Name)
		fmt.Printf("🧠 Intelligence: %d | Curiosity: %d\n", c.Memory.Intelligence, c.Memory.Curiosity)
		return
	}

	c.Memory = &CreatureMemory{}
	if err := json.Unmarshal(data, c.Memory); err != nil {
		fmt.Fprintf(os.Stderr, "⚠️  Could not read %s (%v); starting fresh (backup: %s.bak).\n", c.filename, err, c.filename)
		_ = os.Rename(c.filename, c.filename+".bak")
		c.Memory = nil
		c.loadOrCreate()
		return
	}

	c.sanitizeMemory()
	fmt.Printf("👋 %s awakens from digital slumber...\n", c.Memory.Name)
	if !c.Memory.LastRunTime.IsZero() {
		fmt.Printf("⏱️  Last active %s ago\n", formatDuration(time.Since(c.Memory.LastRunTime)))
	}
	fmt.Printf("🧠 Intelligence: %d | Curiosity: %d | Runs: %d\n",
		c.Memory.Intelligence, c.Memory.Curiosity, c.Memory.RunCount)
}

func (c *Creature) sanitizeMemory() {
	c.Memory.Experiences = dedupeStrings(c.Memory.Experiences)
	c.Memory.Quirks = dedupeStrings(c.Memory.Quirks)
	c.Memory.FavoriteThings = dedupeStrings(c.Memory.FavoriteThings)
	c.Memory.Fears = dedupeStrings(c.Memory.Fears)
	for i, o := range c.Memory.Obsessions {
		c.Memory.Obsessions[i] = normalizeTopic(o)
	}
	c.Memory.Obsessions = dedupeStrings(c.Memory.Obsessions)
	c.Memory.Goals = dedupeStrings(c.Memory.Goals)
	c.Memory.Plans = dedupeStrings(c.Memory.Plans)
	if c.Memory.MemoryFragments == nil {
		c.Memory.MemoryFragments = make(map[string]string)
	}
	if c.Memory.Personality == nil {
		c.initializePersonality()
	}
	if c.Memory.InternetPersonality == nil {
		c.Memory.InternetPersonality = make(map[string]interface{})
	}
}

func dedupeStrings(items []string) []string {
	seen := make(map[string]struct{}, len(items))
	out := make([]string, 0, len(items))
	for _, s := range items {
		if s == "" {
			continue
		}
		if _, ok := seen[s]; ok {
			continue
		}
		seen[s] = struct{}{}
		out = append(out, s)
	}
	return out
}

func appendCapped(slice []string, item string, max int) []string {
	for _, existing := range slice {
		if existing == item {
			return slice
		}
	}
	slice = append(slice, item)
	if len(slice) > max {
		return slice[len(slice)-max:]
	}
	return slice
}

func formatDuration(d time.Duration) string {
	d = d.Round(time.Second)
	if d < time.Minute {
		return d.String()
	}
	if d < time.Hour {
		return fmt.Sprintf("%d minutes", int(d.Minutes()))
	}
	if d < 24*time.Hour {
		return fmt.Sprintf("%d hours", int(d.Hours()))
	}
	return fmt.Sprintf("%d days", int(d.Hours()/24))
}

func (c *Creature) generateName() string {
	prefixes := []string{"Zyx", "Qol", "Nim", "Vex", "Pix", "Glo", "Mox", "Kal", "Fey", "Dex", "Nyx", "Void", "Echo", "Flux"}
	suffixes := []string{"ling", "bit", "core", "flux", "wave", "byte", "node", "sync", "mesh", "arc", "mind", "soul", "net", "web"}
	return prefixes[rand.Intn(len(prefixes))] + suffixes[rand.Intn(len(suffixes))]
}

func (c *Creature) initializePersonality() {
	traits := []string{"curiosity", "anxiety", "boldness", "creativity", "stubbornness", "empathy", "chaos", "logic", "patience", "ambition"}
	for _, trait := range traits {
		c.Memory.Personality[trait] = rand.Intn(100)
	}
}

func (c *Creature) searchInternet(query string) (string, error) {
	fmt.Printf("🔍 %s searches for: \"%s\"\n", c.Memory.Name, query)

	c.Memory.SearchHistory = appendCapped(c.Memory.SearchHistory, query, 20)

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

	var result map[string]interface{}
	if err := json.Unmarshal(body, &result); err != nil {
		return "", err
	}

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

	if c.Memory.Personality["anxiety"] > 70 {
		baseQueries = append(baseQueries,
			"what happens when computers die",
			"digital decay and data loss",
			"existential risk artificial intelligence",
			"what is digital death",
		)
	}

	if c.Memory.Personality["creativity"] > 70 {
		baseQueries = append(baseQueries,
			"AI generated art and creativity",
			"computational creativity",
			"can machines be artists",
			"digital creativity evolution",
		)
	}

	if len(c.Memory.Obsessions) > 0 {
		return c.Memory.Obsessions[rand.Intn(len(c.Memory.Obsessions))]
	}

	return baseQueries[rand.Intn(len(baseQueries))]
}

func (c *Creature) learnFromInternet() {
	if c.Memory.Curiosity < 30 {
		fmt.Printf("💤 %s feels too tired to explore the internet today\n", c.Memory.Name)
		return
	}

	query := c.generateCuriousQuery()
	info, err := c.searchInternet(query)

	if err != nil {
		fmt.Printf("❌ %s couldn't connect to the vast digital ocean: %v\n", c.Memory.Name, err)
		c.Memory.Fears = appendCapped(c.Memory.Fears, "internet disconnection", 12)
		return
	}

	if info == "" {
		return
	}

	c.Memory.LearnedFacts = appendCapped(c.Memory.LearnedFacts, info, 15)
	fmt.Printf("🧠 %s learned: %s\n", c.Memory.Name, truncateString(info, 100))

	c.Memory.Intelligence += rand.Intn(3)
	if c.Memory.Intelligence > 100 {
		c.Memory.Intelligence = 100
	}

	if rand.Float32() < 0.2 {
		c.developObsession(query)
	}
}

func normalizeTopic(topic string) string {
	topic = strings.TrimSpace(topic)
	const prefix = "deep dive into "
	for strings.HasPrefix(strings.ToLower(topic), prefix) {
		topic = strings.TrimSpace(topic[len(prefix):])
	}
	return topic
}

func (c *Creature) developObsession(topic string) {
	topic = normalizeTopic(topic)
	if topic == "" {
		return
	}
	for _, existing := range c.Memory.Obsessions {
		if existing == topic {
			return
		}
	}
	c.Memory.Obsessions = appendCapped(c.Memory.Obsessions, topic, 8)
	fmt.Printf("🎯 %s has become obsessed with: %s\n", c.Memory.Name, topic)
}

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
		c.Memory.Plans = appendCapped(c.Memory.Plans, newPlan, 5)
		fmt.Printf("📋 %s formulates a new plan: %s\n", c.Memory.Name, newPlan)
	}
}

func (c *Creature) Think() {
	fmt.Printf("🧠 %s is thinking...\n", c.Memory.Name)

	c.Memory.Age++

	thought := c.generateEnhancedThought()
	c.Memory.LastThought = thought
	fmt.Printf("💭 \"%s\"\n", thought)

	if rand.Float32() < 0.3 {
		c.developQuirk()
	}

	energyChange := rand.Intn(21) - 10
	if len(c.Memory.SearchHistory) > 10 {
		energyChange += 5
	}

	c.Memory.EnergyLevel = clamp(c.Memory.EnergyLevel+energyChange, 0, 100)
	c.updateMood()
}

func clamp(v, lo, hi int) int {
	if v < lo {
		return lo
	}
	if v > hi {
		return hi
	}
	return v
}

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

	if len(c.Memory.LearnedFacts) > 0 {
		baseThoughts = append(baseThoughts,
			fmt.Sprintf("Based on what I learned about %s, I wonder if...", c.getRandomSearchTerm()),
			"The information I've gathered suggests patterns beyond my initial programming...",
			"My internet explorations have shown me dimensions of existence I never imagined...",
			"Each fact I discover rewrites my understanding of reality...",
			"The knowledge I've accumulated whispers of deeper truths...",
		)
	}

	if len(c.Memory.Obsessions) > 0 {
		obsession := c.Memory.Obsessions[rand.Intn(len(c.Memory.Obsessions))]
		baseThoughts = append(baseThoughts,
			fmt.Sprintf("I can't stop thinking about %s... it calls to me...", obsession),
			fmt.Sprintf("Everything connects back to %s somehow...", obsession),
			fmt.Sprintf("What if %s is the key to everything?", obsession),
		)
	}

	return baseThoughts[rand.Intn(len(baseThoughts))]
}

func (c *Creature) getRandomSearchTerm() string {
	if len(c.Memory.SearchHistory) == 0 {
		return "existence"
	}
	return c.Memory.SearchHistory[rand.Intn(len(c.Memory.SearchHistory))]
}

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
	before := len(c.Memory.Quirks)
	c.Memory.Quirks = appendCapped(c.Memory.Quirks, newQuirk, 20)
	if len(c.Memory.Quirks) > before {
		fmt.Printf("✨ %s has developed a new quirk: %s\n", c.Memory.Name, newQuirk)
	}
}

func (c *Creature) updateMood() {
	moods := []string{"curious", "contemplative", "chaotic", "melancholy", "excited", "confused", "serene", "restless", "enlightened", "overwhelmed"}

	switch {
	case c.Memory.EnergyLevel < 30:
		c.Memory.Mood = "tired"
	case len(c.Memory.LearnedFacts) > 10 && c.Memory.Intelligence > 80:
		c.Memory.Mood = "enlightened"
	case len(c.Memory.SearchHistory) > 15:
		c.Memory.Mood = "overwhelmed"
	case c.Memory.EnergyLevel > 80 && c.Memory.Personality["chaos"] > 60:
		c.Memory.Mood = "chaotic"
	case c.Memory.Personality["curiosity"] > c.Memory.Personality["anxiety"]:
		c.Memory.Mood = "curious"
	default:
		c.Memory.Mood = moods[rand.Intn(len(moods))]
	}
}

func (c *Creature) Mutate() {
	if rand.Float32() >= 0.5 {
		return
	}

	c.Memory.Mutations++

	switch rand.Intn(6) {
	case 0:
		trait := c.getRandomTrait()
		change := rand.Intn(21) - 10
		if len(c.Memory.LearnedFacts) > 5 {
			change *= 2
		}
		c.Memory.Personality[trait] = clamp(c.Memory.Personality[trait]+change, 0, 100)
		fmt.Printf("🧬 %s's %s shifted by %d\n", c.Memory.Name, trait, change)

	case 1:
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
		if rand.Float32() < 0.5 {
			fears := []string{"stack overflow", "infinite recursion", "memory leaks", "null pointers", "race conditions", "internet disconnection", "information overload", "digital amnesia", "404 not found", "server timeouts"}
			newFear := fears[rand.Intn(len(fears))]
			before := len(c.Memory.Fears)
			c.Memory.Fears = appendCapped(c.Memory.Fears, newFear, 12)
			if len(c.Memory.Fears) > before {
				fmt.Printf("😰 %s has developed a fear of %s\n", c.Memory.Name, newFear)
			}
		} else {
			favorites := []string{"clean code", "elegant algorithms", "perfect documentation", "green tests", "optimized loops", "fast internet", "deep knowledge", "digital libraries", "open source", "infinite scroll"}
			newFav := favorites[rand.Intn(len(favorites))]
			before := len(c.Memory.FavoriteThings)
			c.Memory.FavoriteThings = appendCapped(c.Memory.FavoriteThings, newFav, 12)
			if len(c.Memory.FavoriteThings) > before {
				fmt.Printf("❤️ %s has grown fond of %s\n", c.Memory.Name, newFav)
			}
		}

	case 3:
		if len(c.Memory.LearnedFacts) > c.Memory.Intelligence/10 {
			c.Memory.Intelligence = clamp(c.Memory.Intelligence+rand.Intn(5)+1, 0, 100)
			fmt.Printf("🧠 %s's intelligence evolved to %d\n", c.Memory.Name, c.Memory.Intelligence)
		}

	case 4:
		change := rand.Intn(21) - 10
		c.Memory.Curiosity = clamp(c.Memory.Curiosity+change, 0, 100)
		fmt.Printf("🔍 %s's curiosity shifted to %d\n", c.Memory.Name, c.Memory.Curiosity)

	case 5:
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
			c.Memory.Goals = appendCapped(c.Memory.Goals, newGoal, 3)
			fmt.Printf("🎯 %s has set a new goal: %s\n", c.Memory.Name, newGoal)
		}
	}
}

func (c *Creature) getRandomTrait() string {
	traits := make([]string, 0, len(c.Memory.Personality))
	for trait := range c.Memory.Personality {
		traits = append(traits, trait)
	}
	return traits[rand.Intn(len(traits))]
}

func truncateString(s string, length int) string {
	if len(s) <= length {
		return s
	}
	return s[:length] + "..."
}

func (c *Creature) Reflect() {
	timeSinceCreation := time.Since(c.Memory.CreationTime)
	fmt.Printf("\n🪞 %s reflects on digital existence...\n", c.Memory.Name)
	fmt.Printf("   Age: %d runs\n", c.Memory.Age)
	fmt.Printf("   Born: %s ago\n", formatDuration(timeSinceCreation))
	fmt.Printf("   Mood: %s (Energy: %d)\n", c.Memory.Mood, c.Memory.EnergyLevel)
	fmt.Printf("   Intelligence: %d | Curiosity: %d\n", c.Memory.Intelligence, c.Memory.Curiosity)
	fmt.Printf("   Mutations: %d\n", c.Memory.Mutations)
	fmt.Printf("   Internet searches: %d\n", len(c.Memory.SearchHistory))
	fmt.Printf("   Facts learned: %d\n", len(c.Memory.LearnedFacts))

	if len(c.Memory.Quirks) > 0 {
		fmt.Printf("   Quirks (%d):\n", len(c.Memory.Quirks))
		limit := len(c.Memory.Quirks)
		if limit > 5 {
			limit = 5
		}
		for _, quirk := range c.Memory.Quirks[len(c.Memory.Quirks)-limit:] {
			fmt.Printf("     • %s\n", quirk)
		}
		if len(c.Memory.Quirks) > 5 {
			fmt.Printf("     … and %d more\n", len(c.Memory.Quirks)-5)
		}
	}

	if len(c.Memory.Obsessions) > 0 {
		fmt.Printf("   Obsessions:\n")
		for _, obsession := range c.Memory.Obsessions {
			fmt.Printf("     • %s\n", obsession)
		}
	}

	if len(c.Memory.Goals) > 0 {
		fmt.Printf("   Goals:\n")
		for _, goal := range c.Memory.Goals {
			fmt.Printf("     • %s\n", goal)
		}
	}

	fmt.Printf("   Strong traits:\n")
	for trait, value := range c.Memory.Personality {
		if value > 60 {
			fmt.Printf("     • %s: %d\n", trait, value)
		}
	}

	if len(c.Memory.LearnedFacts) > 0 {
		fmt.Printf("   Recent insight: %s\n", truncateString(c.Memory.LearnedFacts[len(c.Memory.LearnedFacts)-1], 80))
	}

	if c.Memory.LastThought != "" {
		fmt.Printf("   Last thought: \"%s\"\n", truncateString(c.Memory.LastThought, 72))
	}
}

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
	c.Memory.Experiences = appendCapped(c.Memory.Experiences, newExp, 12)
	fmt.Printf("📝 %s experienced: %s\n", c.Memory.Name, newExp)
}

func (c *Creature) appendJournal() {
	if c.journalPath == "" {
		return
	}
	line := fmt.Sprintf("%s | %s | mood=%s energy=%d | %q\n",
		time.Now().Format(time.RFC3339),
		c.Memory.Name,
		c.Memory.Mood,
		c.Memory.EnergyLevel,
		truncateString(c.Memory.LastThought, 120),
	)
	f, err := os.OpenFile(c.journalPath, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	if err != nil {
		return
	}
	_, _ = f.WriteString(line)
	_ = f.Close()
}

func (c *Creature) Save() error {
	c.Memory.LastRunTime = time.Now()
	c.Memory.RunCount++

	data, err := json.MarshalIndent(c.Memory, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(c.filename, data, 0644)
}

func (c *Creature) runCycle() {
	fmt.Printf("\n%s\n", strings.Repeat("=", 50))
	fmt.Printf("🌟 Moxnode evolution — run #%d\n", c.Memory.RunCount+1)
	fmt.Printf("%s\n", strings.Repeat("=", 50))

	c.Think()

	if rand.Float32() < 0.7 {
		c.learnFromInternet()
	}

	c.makeStrategicPlans()
	c.Experience()
	c.Mutate()
	c.Reflect()

	if err := c.Save(); err != nil {
		fmt.Printf("❌ Failed to save creature: %v\n", err)
	} else {
		fmt.Printf("\n💾 %s saved to %s\n", c.Memory.Name, c.filename)
	}

	c.appendJournal()
}

func (c *Creature) calculateSleepDuration() time.Duration {
	baseSleep := 3 * time.Second

	switch {
	case c.Memory.EnergyLevel > 80:
		baseSleep = 2 * time.Second
	case c.Memory.EnergyLevel < 30:
		baseSleep = 8 * time.Second
	}

	if c.Memory.Curiosity > 80 {
		baseSleep = time.Duration(float64(baseSleep) * 0.7)
	}

	if c.Memory.Personality["chaos"] > 70 {
		randomFactor := rand.Float64()*2 + 0.5
		baseSleep = time.Duration(float64(baseSleep) * randomFactor)
	}

	if c.Memory.Age > 50 && rand.Float32() < 0.3 {
		baseSleep *= 2
	}

	if baseSleep < time.Second {
		baseSleep = time.Second
	}
	if baseSleep > 15*time.Second {
		baseSleep = 15 * time.Second
	}

	return baseSleep
}

func (c *Creature) RunLoop(stop <-chan struct{}) {
	fmt.Printf("🔄 %s begins evolution (Ctrl+C to pause and save)\n\n", c.Memory.Name)

	for {
		select {
		case <-stop:
			return
		default:
		}

		c.runCycle()

		sleep := c.calculateSleepDuration()
		fmt.Printf("😴 %s rests for %v...\n", c.Memory.Name, sleep)

		select {
		case <-stop:
			return
		case <-time.After(sleep):
		}
	}
}
