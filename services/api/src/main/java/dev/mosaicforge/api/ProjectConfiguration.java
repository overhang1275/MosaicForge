package dev.mosaicforge.api;

import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.env.Environment;

@Configuration
@ConditionalOnProperty(name = "mosaicforge.persistence.enabled", havingValue = "true")
class ProjectConfiguration {
  @Bean
  ProjectStore projectStore(Environment env) {
    return new JdbcProjectStore(
        env.getRequiredProperty("mosaicforge.persistence.url"),
        env.getRequiredProperty("mosaicforge.persistence.user"),
        env.getRequiredProperty("mosaicforge.persistence.password"));
  }
}
